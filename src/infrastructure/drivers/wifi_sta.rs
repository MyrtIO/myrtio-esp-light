use embassy_executor::Spawner;
use embassy_net::{DhcpConfig, Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::WIFI;
#[cfg(feature = "log")]
use esp_println::println;
use esp_radio::wifi::{
    AuthMethod,
    ClientConfig,
    Config,
    ModeConfig,
    WifiController,
    WifiDevice,
    WifiEvent,
    WifiStaState,
};
use heapless::String;
use static_cell::make_static;

use super::random::get_seed;

/// Maximum length of the hostname
const MAX_HOSTNAME_LEN: usize = 32;

const MAX_NETWORK_CONNECTIONS: usize = 6;

/// Type alias for the hostname
pub type Hostname = heapless::String<MAX_HOSTNAME_LEN>;

/// Start the Wi-Fi STA (Station) mode
///
/// It connects to the `WiFi` network and waits for the connection to be established.
/// If the connection is lost, it tries to reconnect.
pub async fn start_wifi_sta(
    spawner: Spawner,
    wifi_device: WIFI<'static>,
    ssid: String<32>,
    password: String<64>,
    hostname: Hostname,
) -> Stack<'static> {
    let esp_radio_ctrl = &*make_static!(esp_radio::init().unwrap());
    let (controller, interfaces) =
        esp_radio::wifi::new(esp_radio_ctrl, wifi_device, Config::default())
            .unwrap();
    let mut dhcp_config = DhcpConfig::default();
    dhcp_config.hostname = Some(hostname);

    let net_config = embassy_net::Config::dhcpv4(dhcp_config);

    let network_resources =
        make_static!(StackResources::<{ MAX_NETWORK_CONNECTIONS }>::new());
    let (stack, runner) =
        embassy_net::new(interfaces.sta, net_config, network_resources, get_seed());

    spawner
        .spawn(wifi_connection_task(controller, ssid, password))
        .ok();
    spawner.spawn(network_runner_task(runner)).ok();

    wait_for_connection(stack).await;

    stack
}

/// Background task for connecting to the `WiFi` network and reconnecting if needed
#[embassy_executor::task]
pub async fn wifi_connection_task(
    mut controller: WifiController<'static>,
    ssid: String<32>,
    password: String<64>,
) {
    loop {
        // Wait until we're no longer connected
        if esp_radio::wifi::sta_state() == WifiStaState::Connected {
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(2000)).await;
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = if password.is_empty() {
                ClientConfig::default()
                    .with_ssid(ssid.as_str().into())
                    .with_auth_method(AuthMethod::None)
            } else {
                ClientConfig::default()
                    .with_ssid(ssid.as_str().into())
                    .with_password(password.as_str().into())
            };
            let mode_config = ModeConfig::Client(client_config);
            controller.set_config(&mode_config).unwrap();
            controller.start_async().await.unwrap();
        }

        #[cfg(feature = "log")]
        println!("network: connecting");
        if let Err(_e) = controller.connect_async().await {
            #[cfg(feature = "log")]
            println!("network: error connecting: {:?}", _e);
            Timer::after(Duration::from_millis(5000)).await;
        }
    }
}

/// Background task for running the network stack
#[embassy_executor::task]
pub async fn network_runner_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}

/// Wait for full network connectivity (link + IP address)
/// Returns the obtained IPv4 configuration
async fn wait_for_connection(stack: Stack<'_>) -> embassy_net::StaticConfigV4 {
    // Wait for the network link to become active
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(100)).await;
    }

    // Wait for the network stack to obtain an IPv4 address via DHCP
    loop {
        if let Some(config) = stack.config_v4() {
            return config;
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}
