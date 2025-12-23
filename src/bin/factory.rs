//! Factory Firmware
//!
//! This binary provides a provisioning interface for initial device setup:
//! - Starts a Wi-Fi Access Point (MyrtIO-Setup-XXXX)
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
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::peripherals::GPIO2;
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};
use esp_println::println;
use esp_storage::FlashStorage;
use static_cell::StaticCell;

use myrtio_esp_light::app::ConfigurationUsecases;
use myrtio_esp_light::config;
use myrtio_esp_light::controllers::init_factory_controllers;
use myrtio_esp_light::infrastructure::drivers::init_network_stack_ap;
use myrtio_esp_light::infrastructure::repositories::AppPersistentStorage;
use myrtio_esp_light::infrastructure::tasks::factory::{
    blink_task, dhcp_server_task, factory_network_runner_task, factory_wifi_ap_task,
    http_server_task,
};
use myrtio_esp_light::mk_static;

esp_bootloader_esp_idf::esp_app_desc!();

static FLASH_STORAGE: StaticCell<FlashStorage<'static>> = StaticCell::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    println!("=================================");
    println!("  MyrtIO Factory Firmware");
    println!("=================================");

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

    // Spawn blink task
    spawner.spawn(blink_task(peripherals.GPIO2)).ok();

    // Initialize flash storage (shared between HTTP server for config and OTA)
    let flash = FLASH_STORAGE.init(FlashStorage::new(peripherals.FLASH));
    let flash_ptr = flash as *mut FlashStorage<'static>;

    // Initialize network stack for AP mode
    let (stack, runner, controller) = init_network_stack_ap(peripherals.WIFI);

    // Spawn WiFi AP task
    spawner.spawn(factory_wifi_ap_task(controller)).ok();

    // Spawn network runner
    spawner.spawn(factory_network_runner_task(runner)).ok();

    // Wait for the network link to be up
    println!("Waiting for AP to be ready...");
    loop {
        if stack.is_link_up() {
            break;
        }
        embassy_time::Timer::after(Duration::from_millis(100)).await;
    }
    println!("AP link is up!");

    // Additional delay for stability
    embassy_time::Timer::after(Duration::from_millis(500)).await;

    // Spawn DHCP server
    spawner.spawn(dhcp_server_task(stack)).ok();

    // Initialize configuration usecases and http handler
    let storage = AppPersistentStorage::new(flash_ptr, config::CONFIGURATION_PARTITION_OFFSET);
    let usecases = mk_static!(
        ConfigurationUsecases<AppPersistentStorage>,
        ConfigurationUsecases::new(storage)
    );
    let handler = init_factory_controllers(usecases, flash_ptr);

    // Spawn HTTP server
    spawner.spawn(http_server_task(stack, handler)).ok();

    println!("Factory firmware ready!");
    println!("Connect to WiFi: MyrtIO-Setup-XXXX");
    println!("Open http://192.168.4.1 in browser");

    // Main loop - just keep running
    loop {
        embassy_time::Timer::after(Duration::from_secs(60)).await;
    }
}
