//! Factory Firmware
//!
//! This binary provides a provisioning interface for initial device setup:
//! - Starts a Wi-Fi Access Point
//! - Runs a DHCP server for clients
//! - Serves an HTTP configuration page on 192.168.4.1
//! - Allows configuration of `WiFi`, MQTT, and LED settings
//! - Allows uploading OTA firmware to the next partition

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Duration;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
    peripherals,
    timer::timg::TimerGroup,
};
use esp_println::println;
use myrtio_esp_light::{
    app::{ConfigurationUsecases, FirmwareUsecases},
    config,
    controllers::factory::{self, FactoryHttpController, init_factory_controllers},
    domain::{
        entity::LightState,
        ports::{
            LightConfigChanger as _,
            LightStateChanger as _,
            PersistentDataReader as _,
        },
    },
    infrastructure::{
        adapters::{self},
        drivers::{self, WifiApConfig},
        services::{self},
        types::{ConfigurationUsecasesImpl, FirmwareUsecasesImpl},
    },
    mk_static,
};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    // Initialize hardware
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Allocate heap memory (64 + 32 KB)
    esp_alloc::heap_allocator!(
        #[unsafe(link_section = ".dram2_uninit")] size: 64 * 1024
    );
    esp_alloc::heap_allocator!(size: 32 * 1024);

    // Start RTOS
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    // Start blinker
    spawner.must_spawn(blink_led_task(peripherals.GPIO2));

    // Initialize Services
    services::init_flash_storage(peripherals.FLASH).await;
    let firmware_service = services::init_firmware(spawner);
    let persistence_service = services::init_persistence(spawner);

    // Initialize light service
    let mut light_service = services::init_light(
        spawner,
        peripherals.RMT,
        myrtio_esp_light::led_gpio!(peripherals),
    );

    // Configure light
    let (_, device_config) = persistence_service
        .read_persistent_data()
        .unwrap_or_default();
    light_service
        .apply_light_intent(LightState::default().into())
        .unwrap();
    light_service.set_config(device_config.light).unwrap();

    // Initialize usecases
    let configuration = mk_static!(
        ConfigurationUsecasesImpl,
        ConfigurationUsecases::new(persistence_service, light_service)
    );
    let firmware_usecases = mk_static!(
        FirmwareUsecasesImpl,
        FirmwareUsecases::new(firmware_service)
    );

    // Bind boot button
    adapters::bind_boot_button(
        peripherals.IO_MUX,
        peripherals.GPIO0,
        factory::handle_boot_button_click,
    );

    let stack = drivers::start_wifi_ap(
        spawner,
        peripherals.WIFI,
        WifiApConfig {
            ssid: config::access_point_name(),
            ip_address: config::FACTORY_AP_IP_ADDRESS,
            gateway: config::FACTORY_AP_GATEWAY,
            prefix_len: config::FACTORY_AP_PREFIX_LEN,
        },
    )
    .await;

    let handler = mk_static!(
        FactoryHttpController,
        init_factory_controllers(configuration, firmware_usecases).await
    );

    println!("Factory firmware ready!");
    println!("Connect to WiFi: MyrtIO-Setup-XXXX");
    println!("Open http://192.168.4.1 in browser");

    adapters::run_http_server(stack, handler).await;

    unreachable!();
}

#[embassy_executor::task]
async fn blink_led_task(gpio: peripherals::GPIO2<'static>) {
    let mut pin = Output::new(gpio, Level::High, OutputConfig::default());
    loop {
        pin.set_high();
        embassy_time::Timer::after(Duration::from_millis(500)).await;
        pin.set_low();
        embassy_time::Timer::after(Duration::from_millis(500)).await;
    }
}
