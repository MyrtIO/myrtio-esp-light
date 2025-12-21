//! Factory Wi-Fi AP task
//!
//! Configures and maintains the Wi-Fi controller in Access Point mode
//! for the factory provisioning interface.

use embassy_net::Runner;
use esp_println::println;
use esp_radio::wifi::{AccessPointConfig, AuthMethod, ModeConfig, WifiController, WifiDevice};

/// Default SSID prefix for the factory AP
const AP_SSID_PREFIX: &str = "MyrtIO-Setup";

/// Background task for running the Wi-Fi AP
///
/// Configures the controller in AP mode with an open network.
/// SSID is derived from the chip ID for uniqueness.
#[embassy_executor::task]
pub async fn factory_wifi_ap_task(mut controller: WifiController<'static>) {
    // Get chip ID for unique SSID
    let chip_id = get_chip_id();
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
pub async fn factory_network_runner_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}

/// Get the chip ID for SSID generation
fn get_chip_id() -> u32 {
    // Use the last 4 bytes of the MAC address as chip ID
    let mac = esp_hal::efuse::Efuse::mac_address();
    u32::from_be_bytes([mac[2], mac[3], mac[4], mac[5]])
}

/// Format the SSID with chip ID suffix
fn format_ssid(chip_id: u32) -> heapless::String<32> {
    use core::fmt::Write;
    let mut ssid = heapless::String::<32>::new();
    let _ = write!(ssid, "{}-{:04X}", AP_SSID_PREFIX, chip_id & 0xFFFF);
    ssid
}

