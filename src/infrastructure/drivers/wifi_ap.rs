use embassy_executor::Spawner;
use embassy_net::{
    Ipv4Address,
    Ipv4Cidr,
    Runner,
    Stack,
    StackResources,
    StaticConfigV4,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::WIFI;
#[cfg(feature = "log")]
use esp_println::println;
use esp_radio::wifi::{
    AccessPointConfig,
    AuthMethod,
    Config,
    ModeConfig,
    WifiController,
    WifiDevice,
};
use static_cell::make_static;

use super::random::get_seed;
use crate::core::net::dhcp::{
    DHCP_ACK,
    DHCP_DISCOVER,
    DHCP_OFFER,
    DHCP_REQUEST,
    allocate_ip,
    build_dhcp_response,
    parse_dhcp_request,
};

/// DHCP server and client ports
const DHCP_SERVER_PORT: u16 = 67;
const DHCP_CLIENT_PORT: u16 = 68;

const MAX_CONNECTIONS: usize = 6;

pub struct WifiApConfig {
    pub ssid: heapless::String<32>,
    pub ip_address: Ipv4Address,
    pub gateway: Ipv4Address,
    pub prefix_len: u8,
}

/// Initialize the network stack for AP (Access Point) mode.
///
/// Uses a static IP configuration (192.168.4.1/24) suitable for a captive portal.
pub async fn start_wifi_ap(
    spawner: Spawner,
    wifi_device: WIFI<'static>,
    config: WifiApConfig,
) -> Stack<'static> {
    let esp_radio_ctrl = &*make_static!(esp_radio::init().unwrap());
    let wifi_config = Config::default();
    let (controller, interfaces) =
        esp_radio::wifi::new(esp_radio_ctrl, wifi_device, wifi_config).unwrap();

    // Static IP configuration for AP mode
    let static_config = StaticConfigV4 {
        address: Ipv4Cidr::new(config.ip_address, config.prefix_len),
        gateway: Some(config.gateway),
        dns_servers: heapless::Vec::default(),
    };
    let net_config = embassy_net::Config::ipv4_static(static_config);

    let network_resources = make_static!(StackResources::<MAX_CONNECTIONS>::new());
    let (stack, runner) =
        embassy_net::new(interfaces.ap, net_config, network_resources, get_seed());

    spawner
        .spawn(factory_wifi_ap_task(controller, config.ssid))
        .ok();
    spawner.spawn(factory_network_runner_task(runner)).ok();

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(100)).await;
    }
    // Give some extra time
    Timer::after(Duration::from_millis(100)).await;

    spawner
        .spawn(dhcp_server_task(stack, config.ip_address))
        .ok();

    stack
}

/// Background task for running the Wi-Fi AP
///
/// Configures the controller in AP mode with an open network.
/// SSID is derived from the chip ID for uniqueness.
#[embassy_executor::task]
pub async fn factory_wifi_ap_task(
    mut controller: WifiController<'static>,
    ssid: heapless::String<32>,
) {
    #[cfg(feature = "log")]
    println!("factory_wifi: starting AP with SSID '{}'", ssid.as_str());

    let ap_config = AccessPointConfig::default()
        .with_ssid(ssid.as_str().into())
        .with_auth_method(AuthMethod::None);

    let mode_config = ModeConfig::AccessPoint(ap_config);
    controller.set_config(&mode_config).unwrap();
    controller.start_async().await.unwrap();

    #[cfg(feature = "log")]
    println!("factory_wifi: AP started");

    // Keep the AP running
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(60)).await;
    }
}

/// DHCP server task
///
/// Listens for DHCP discover/request messages and responds with offers/acks.
/// Uses a stateless allocation strategy based on client MAC address.
#[embassy_executor::task]
pub async fn dhcp_server_task(stack: Stack<'static>, ap_ip_address: Ipv4Address) {
    #[cfg(feature = "log")]
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

    if let Err(_e) = socket.bind(DHCP_SERVER_PORT) {
        #[cfg(feature = "log")]
        println!(
            "dhcp_server: failed to bind port {}: {:?}",
            DHCP_SERVER_PORT, _e
        );
        return;
    }
    #[cfg(feature = "log")]
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
                        #[cfg(feature = "log")]
                        println!(
                            "dhcp_server: unknown message type {}, ignoring",
                            request.message_type
                        );
                        continue;
                    }
                };

                // Build response
                let response_len = build_dhcp_response(
                    ap_ip_address,
                    &mut packet,
                    &request,
                    offered_ip,
                    response_type,
                );

                // Send to broadcast on client port
                let dest = (Ipv4Address::BROADCAST, DHCP_CLIENT_PORT);
                match socket.send_to(&packet[..response_len], dest).await {
                    Ok(()) => {}
                    Err(_e) => {
                        #[cfg(feature = "log")]
                        println!("dhcp_server: send error: {:?}", _e);
                    }
                }
            }
            Err(_e) => {
                #[cfg(feature = "log")]
                println!("dhcp_server: recv error: {:?}", _e);
            }
        }
    }
}

/// Background task for running the network stack
#[embassy_executor::task]
pub async fn factory_network_runner_task(
    mut runner: Runner<'static, WifiDevice<'static>>,
) {
    runner.run().await;
}
