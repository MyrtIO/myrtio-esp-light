//! DHCP Server Task
//!
//! Embassy task that runs the DHCP server for Factory AP mode.

use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Ipv4Address, Stack};
use esp_println::println;

use crate::infrastructure::services::{
    DHCP_ACK, DHCP_DISCOVER, DHCP_OFFER, DHCP_REQUEST,
    allocate_ip, build_dhcp_response, parse_dhcp_request,
};

/// DHCP server and client ports
const DHCP_SERVER_PORT: u16 = 67;
const DHCP_CLIENT_PORT: u16 = 68;

/// DHCP server task
///
/// Listens for DHCP discover/request messages and responds with offers/acks.
/// Uses a stateless allocation strategy based on client MAC address.
#[embassy_executor::task]
pub async fn dhcp_server_task(stack: Stack<'static>) {
    println!("dhcp_server: starting on port {}", DHCP_SERVER_PORT);

    let mut rx_meta = [PacketMetadata::EMPTY; 8];
    let mut rx_buffer = [0u8; 1024];
    let mut tx_meta = [PacketMetadata::EMPTY; 8];
    let mut tx_buffer = [0u8; 1024];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    if let Err(e) = socket.bind(DHCP_SERVER_PORT) {
        println!("dhcp_server: failed to bind port {}: {:?}", DHCP_SERVER_PORT, e);
        return;
    }
    println!("dhcp_server: bound to port {}, waiting for packets...", DHCP_SERVER_PORT);

    let mut packet = [0u8; 576];

    loop {
        match socket.recv_from(&mut packet).await {
            Ok((len, remote)) => {
                println!("dhcp_server: received {} bytes from {:?}", len, remote);

                // Parse the DHCP request
                let Some(request) = parse_dhcp_request(&packet[..len]) else {
                    println!("dhcp_server: invalid DHCP packet, ignoring");
                    continue;
                };

                println!("dhcp_server: message type = {}", request.message_type);

                let offered_ip = allocate_ip(&request.client_mac);

                let response_type = match request.message_type {
                    DHCP_DISCOVER => {
                        println!(
                            "dhcp_server: DISCOVER from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                            request.client_mac[0], request.client_mac[1], request.client_mac[2],
                            request.client_mac[3], request.client_mac[4], request.client_mac[5]
                        );
                        DHCP_OFFER
                    }
                    DHCP_REQUEST => {
                        println!(
                            "dhcp_server: REQUEST from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} -> {}",
                            request.client_mac[0], request.client_mac[1], request.client_mac[2],
                            request.client_mac[3], request.client_mac[4], request.client_mac[5],
                            offered_ip
                        );
                        DHCP_ACK
                    }
                    _ => {
                        println!("dhcp_server: unknown message type {}, ignoring", request.message_type);
                        continue;
                    }
                };

                // Build response
                let response_len = build_dhcp_response(
                    &mut packet,
                    &request,
                    offered_ip,
                    response_type,
                );

                // Send to broadcast on client port
                let dest = (Ipv4Address::BROADCAST, DHCP_CLIENT_PORT);
                println!("dhcp_server: sending {} bytes to {:?}", response_len, dest);
                match socket.send_to(&packet[..response_len], dest).await {
                    Ok(()) => println!("dhcp_server: response sent successfully"),
                    Err(e) => println!("dhcp_server: send error: {:?}", e),
                }
            }
            Err(e) => {
                println!("dhcp_server: recv error: {:?}", e);
            }
        }
    }
}
