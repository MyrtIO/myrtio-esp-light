#![no_std]

use embassy_net::Stack;
use embassy_time::{Duration, Timer};

/// Configuration for WiFi connection
pub struct WifiConfig<'a> {
    pub ssid: &'a str,
    pub password: &'a str,
}

impl<'a> WifiConfig<'a> {
    pub const fn new(ssid: &'a str, password: &'a str) -> Self {
        Self { ssid, password }
    }
}

/// Wait for the network link to become active
pub async fn wait_for_link(stack: Stack<'_>) {
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

/// Wait for the network stack to obtain an IPv4 address via DHCP
/// Returns the obtained IPv4 configuration
pub async fn wait_for_ip(stack: Stack<'_>) -> embassy_net::StaticConfigV4 {
    loop {
        if let Some(config) = stack.config_v4() {
            return config;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

/// Wait for full network connectivity (link + IP address)
/// Returns the obtained IPv4 configuration
pub async fn wait_for_connection(stack: Stack<'_>) -> embassy_net::StaticConfigV4 {
    wait_for_link(stack).await;
    wait_for_ip(stack).await
}
