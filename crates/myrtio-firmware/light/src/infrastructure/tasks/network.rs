use embassy_net::Runner;
use embassy_time::{Duration, Timer};
use esp_println::println;
use esp_radio::wifi::{
    ClientConfig, ModeConfig, WifiController, WifiDevice, WifiEvent, WifiStaState,
};

use crate::infrastructure::config;

/// Background task for connecting to the `WiFi` network
///
/// It connects to the `WiFi` network and waits for the connection to be established.
/// If the connection is lost, it tries to reconnect.
#[embassy_executor::task]
pub(crate) async fn wifi_connection_task(mut controller: WifiController<'static>) {
    loop {
        // Wait until we're no longer connected
        if esp_radio::wifi::sta_state() == WifiStaState::Connected {
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(2000)).await;
        }
        // Start the controller if it's not started
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(config::WIFI_SSID.into())
                    .with_password(config::WIFI_PASSWORD.into()),
            );
            controller.set_config(&client_config).unwrap();
            controller.start_async().await.unwrap();
        }

        if let Err(e) = controller.connect_async().await {
            println!("Failed to connect to wifi: {e:?}");
            Timer::after(Duration::from_millis(5000)).await;
        }
    }
}

/// Background task for running the network stack
#[embassy_executor::task]
pub(crate) async fn network_runner_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}
