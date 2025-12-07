use embassy_net::{IpAddress, Stack, dns::DnsQueryType};
use embassy_time::{Duration, Timer};

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
pub async fn resolve_host(stack: Stack<'static>, host: &str) -> Result<IpAddress, ()> {
    if let Ok(ip) = host.parse::<embassy_net::Ipv4Address>() {
        return Ok(IpAddress::Ipv4(ip));
    }

    let Ok(addresses) = stack.dns_query(host, DnsQueryType::A).await else {
        return Err(());
    };

    addresses.first().copied().ok_or(())
}
