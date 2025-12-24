//! Factory Wi-Fi AP task
//!
//! Configures and maintains the Wi-Fi controller in Access Point mode
//! for the factory provisioning interface.

use embassy_net::{
    Ipv4Address,
    Runner,
    Stack,
    udp::{PacketMetadata, UdpSocket},
};
use esp_println::println;
use esp_radio::wifi::{
    AccessPointConfig,
    AuthMethod,
    ModeConfig,
    WifiController,
    WifiDevice,
};

use crate::{
    config::hardware_id,
    core::net::dhcp::{
        DHCP_ACK,
        DHCP_DISCOVER,
        DHCP_OFFER,
        DHCP_REQUEST,
        allocate_ip,
        build_dhcp_response,
        parse_dhcp_request,
    },
};

/// Format the SSID with chip ID suffix
fn format_ssid(chip_id: u32) -> heapless::String<32> {
    use core::fmt::Write;
    let mut ssid = heapless::String::<32>::new();
    let _ = write!(ssid, "{}-{:04X}", AP_SSID_PREFIX, chip_id & 0xFFFF);
    ssid
}

/// Default SSID prefix for the factory AP
const AP_SSID_PREFIX: &str = "MyrtIO Светильник";

/// DHCP server and client ports
const DHCP_SERVER_PORT: u16 = 67;
const DHCP_CLIENT_PORT: u16 = 68;

/// Background task for running the Wi-Fi AP
///
/// Configures the controller in AP mode with an open network.
/// SSID is derived from the chip ID for uniqueness.
#[embassy_executor::task]
pub async fn factory_wifi_ap_task(mut controller: WifiController<'static>) {
    // Get chip ID for unique SSID
    let chip_id = hardware_id();
    let ssid = format_ssid(chip_id);

    println!("factory_wifi: starting AP with SSID '{}'", ssid.as_str());

    let ap_config = AccessPointConfig::default()
        .with_ssid(ssid.as_str().into())
        .with_auth_method(AuthMethod::None);

    let mode_config = ModeConfig::AccessPoint(ap_config);
    controller.set_config(&mode_config).unwrap();
    controller.start_async().await.unwrap();

    println!("factory_wifi: AP started");

    // Keep the AP running
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(60)).await;
    }
}

/// Background task for running the network stack
#[embassy_executor::task]
pub async fn factory_network_runner_task(
    mut runner: Runner<'static, WifiDevice<'static>>,
) {
    runner.run().await;
}

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
        println!(
            "dhcp_server: failed to bind port {}: {:?}",
            DHCP_SERVER_PORT, e
        );
        return;
    }
    println!(
        "dhcp_server: bound to port {}, waiting for packets...",
        DHCP_SERVER_PORT
    );

    let mut packet = [0u8; 576];

    loop {
        match socket.recv_from(&mut packet).await {
            Ok((len, _remote)) => {
                // Parse the DHCP request
                let Some(request) = parse_dhcp_request(&packet[..len]) else {
                    continue;
                };

                let offered_ip = allocate_ip(&request.client_mac);

                let response_type = match request.message_type {
                    DHCP_DISCOVER => DHCP_OFFER,
                    DHCP_REQUEST => DHCP_ACK,
                    _ => {
                        println!(
                            "dhcp_server: unknown message type {}, ignoring",
                            request.message_type
                        );
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
                match socket.send_to(&packet[..response_len], dest).await {
                    Ok(()) => {}
                    Err(e) => println!("dhcp_server: send error: {:?}", e),
                }
            }
            Err(e) => {
                println!("dhcp_server: recv error: {:?}", e);
            }
        }
    }
}
