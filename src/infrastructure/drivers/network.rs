use embassy_net::{
    DhcpConfig,
    IpAddress,
    Ipv4Address,
    Ipv4Cidr,
    Runner,
    Stack,
    StackResources,
    StaticConfigV4,
    dns::DnsQueryType,
};
use embassy_time::{Duration, Timer};
use esp_hal::{peripherals::WIFI, rng::Rng};
use esp_radio::wifi::{Config as WifiConfig, WifiController, WifiDevice};
use heapless::{String, Vec};
use static_cell::make_static;

use crate::config::hardware_id;

const MAX_CONNECTIONS: usize = 6;

/// AP mode static IP configuration
pub const AP_IP_ADDRESS: Ipv4Address = Ipv4Address::new(192, 168, 4, 1);
const AP_GATEWAY: Ipv4Address = Ipv4Address::new(192, 168, 4, 1);
const AP_PREFIX_LEN: u8 = 24;

fn format_hostname(hardware_id: u32) -> String<32> {
    use core::fmt::Write;
    let mut hostname = String::<32>::new();
    let _ = write!(hostname, "MyrtIO Light {:04X}", hardware_id);
    hostname
}

pub fn init_network_stack(
    wifi_device: WIFI<'static>,
) -> (
    Stack<'static>,
    Runner<'static, WifiDevice<'static>>,
    WifiController<'static>,
) {
    let esp_radio_ctrl = &*make_static!(esp_radio::init().unwrap());
    let wifi_config = WifiConfig::default();
    let (controller, interfaces) =
        esp_radio::wifi::new(esp_radio_ctrl, wifi_device, wifi_config).unwrap();
    let mut dhcp_config = DhcpConfig::default();
    dhcp_config.hostname = Some(format_hostname(hardware_id()));

    let net_config = embassy_net::Config::dhcpv4(dhcp_config);

    let network_resources = make_static!(StackResources::<MAX_CONNECTIONS>::new());
    let (stack, runner) =
        embassy_net::new(interfaces.sta, net_config, network_resources, get_seed());

    (stack, runner, controller)
}

/// Initialize the network stack for AP (Access Point) mode.
///
/// Uses a static IP configuration (192.168.4.1/24) suitable for a captive portal.
/// Returns the network [`Stack`], [`Runner`], and [`WifiController`].
pub fn init_network_stack_ap(
    wifi_device: WIFI<'static>,
) -> (
    Stack<'static>,
    Runner<'static, WifiDevice<'static>>,
    WifiController<'static>,
) {
    let esp_radio_ctrl = &*make_static!(esp_radio::init().unwrap());
    let wifi_config = WifiConfig::default();
    let (controller, interfaces) =
        esp_radio::wifi::new(esp_radio_ctrl, wifi_device, wifi_config).unwrap();

    // Static IP configuration for AP mode
    let static_config = StaticConfigV4 {
        address: Ipv4Cidr::new(AP_IP_ADDRESS, AP_PREFIX_LEN),
        gateway: Some(AP_GATEWAY),
        dns_servers: Vec::default(),
    };
    let net_config = embassy_net::Config::ipv4_static(static_config);

    let network_resources = make_static!(StackResources::<MAX_CONNECTIONS>::new());
    let (stack, runner) =
        embassy_net::new(interfaces.ap, net_config, network_resources, get_seed());

    (stack, runner, controller)
}

fn get_seed() -> u64 {
    let rng = Rng::new();
    u64::from(rng.random()) << 32 | u64::from(rng.random())
}

/// Wait for the network link to become active
async fn wait_for_link(stack: Stack<'_>) {
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}

/// Wait for the network stack to obtain an IPv4 address via DHCP
/// Returns the obtained IPv4 configuration
async fn wait_for_ip(stack: Stack<'_>) -> embassy_net::StaticConfigV4 {
    loop {
        if let Some(config) = stack.config_v4() {
            return config;
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}

/// Wait for full network connectivity (link + IP address)
/// Returns the obtained IPv4 configuration
pub async fn wait_for_connection(stack: Stack<'_>) -> embassy_net::StaticConfigV4 {
    wait_for_link(stack).await;
    wait_for_ip(stack).await
}

/// Resolves a hostname to an IP address
pub(crate) async fn resolve_host(
    stack: Stack<'static>,
    host: &str,
) -> Result<IpAddress, ()> {
    if let Ok(ip) = host.parse::<embassy_net::Ipv4Address>() {
        return Ok(IpAddress::Ipv4(ip));
    }

    let Ok(addresses) = stack.dns_query(host, DnsQueryType::A).await else {
        return Err(());
    };

    addresses.first().copied().ok_or(())
}
