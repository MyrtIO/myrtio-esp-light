#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_net::{Runner, StackResources};
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, rng::Rng, timer::timg::TimerGroup};
use esp_println::println;
use esp_radio::{
    Controller,
    wifi::{
        ClientConfig, ModeConfig, ScanConfig, WifiController, WifiDevice, WifiEvent, WifiStaState,
    },
};
use myrtio_light_composer::{
    Command, CommandChannel, CommandSender, EffectSlot, LightEngine, effect::RainbowEffect,
};

use rust_mqtt::{
    client::{client::MqttClient, client_config::ClientConfig},
    packet::v5::reason_codes::ReasonCode,
    utils::rng_generator::CountingRng,
};

mod hardware;
use hardware::EspLedDriver;

pub mod config;

esp_bootloader_esp_idf::esp_app_desc!();

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

/// Static command channel for light engine
static LIGHT_CHANNEL: CommandChannel<{ config::NUM_LEDS }> = Channel::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    // Initialize hardware
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Allocate heap memory
    esp_alloc::heap_allocator!(
        #[unsafe(link_section = ".dram2_uninit")] size: 64 * 1024
    );
    esp_alloc::heap_allocator!(size: 32 * 1024);

    // Start rtos timer
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    // Initialize network stack
    let esp_radio_ctrl = &*mk_static!(Controller<'static>, esp_radio::init().unwrap());
    let (controller, interfaces) =
        esp_radio::wifi::new(&esp_radio_ctrl, peripherals.WIFI, Default::default()).unwrap();

    let wifi_interface = interfaces.sta;

    let config = embassy_net::Config::dhcpv4(Default::default());

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    // Spawn background tasks
    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(runner)).ok();

    let config = myrtio_wifi::wait_for_connection(stack).await;
    println!("Got IP: {}", config.address);

    // Initialize light engine with command channel
    let driver = EspLedDriver::<{ config::NUM_LEDS }>::new(peripherals.RMT, peripherals.GPIO25);
    let mut engine = LightEngine::new(driver, LIGHT_CHANNEL.receiver());

    // Set initial effect
    engine.switch_effect_instant(EffectSlot::Rainbow(RainbowEffect::default()));

    // Spawn brightness cycling demo task
    spawner.spawn(brightness_demo(LIGHT_CHANNEL.sender())).ok();

    loop {
        engine.tick().await;
    }
}

/// Demo task that cycles through brightness values
#[embassy_executor::task]
async fn brightness_demo(sender: CommandSender<{ config::NUM_LEDS }>) {
    // Brightness values to cycle through: 0 -> 128 -> 255 -> 128 -> repeat
    let brightness_values: [u8; 4] = [0, 10, 50, 10];
    let mut index = 0;

    loop {
        let brightness = brightness_values[index];
        println!("Setting brightness to {}", brightness);

        // Send brightness command with 500ms transition
        sender
            .send(Command::SetBrightness {
                brightness,
                duration: Duration::from_millis(500),
            })
            .await;

        // Wait 2 seconds before next change
        Timer::after(Duration::from_secs(2)).await;

        // Move to next value
        index = (index + 1) % brightness_values.len();
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    loop {
        match esp_radio::wifi::sta_state() {
            WifiStaState::Connected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(config::WIFI_SSID.into())
                    .with_password(config::WIFI_PASSWORD.into()),
            );
            controller.set_config(&client_config).unwrap();
            controller.start_async().await.unwrap();
        }

        match controller.connect_async().await {
            Ok(_) => println!("Wifi connected, "),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
