use core::str::FromStr;

use heapless::String;

use embassy_net::{DhcpConfig, IpAddress, Runner, Stack, StackResources, dns::DnsQueryType};
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::WIFI;
use esp_hal::rng::Rng;
use esp_radio::wifi::{Config as WifiConfig, WifiController, WifiDevice};

use static_cell::make_static;

use crate::infrastructure::config;

const MAX_CONNECTIONS: usize = 6;

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
    let hostname = String::from_str(config::DEVICE.hostname).expect("Invalid hostname");
    dhcp_config.hostname = Some(hostname);

    let net_config = embassy_net::Config::dhcpv4(dhcp_config);

    let network_resources = make_static!(StackResources::<MAX_CONNECTIONS>::new());
    let (stack, runner) =
        embassy_net::new(interfaces.sta, net_config, network_resources, get_seed());

    (stack, runner, controller)
}

fn get_seed() -> u64 {
    let rng = Rng::new();
    u64::from(rng.random()) << 32 | u64::from(rng.random())
}

/// Wait for the network link to become active
pub async fn wait_for_link(stack: Stack<'_>) {
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}

/// Wait for the network stack to obtain an IPv4 address via DHCP
/// Returns the obtained IPv4 configuration
pub async fn wait_for_ip(stack: Stack<'_>) -> embassy_net::StaticConfigV4 {
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
pub(crate) async fn resolve_host(stack: Stack<'static>, host: &str) -> Result<IpAddress, ()> {
    if let Ok(ip) = host.parse::<embassy_net::Ipv4Address>() {
        return Ok(IpAddress::Ipv4(ip));
    }

    let Ok(addresses) = stack.dns_query(host, DnsQueryType::A).await else {
        return Err(());
    };

    addresses.first().copied().ok_or(())
}
