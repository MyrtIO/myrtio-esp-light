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
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_storage::FlashStorage;
use myrtio_esp_light::{
    app::{ConfigurationUsecases, FirmwareUsecases},
    controllers::{factory::{handle_boot_button_click, init_factory_controllers}, },
    domain::{
        dto::LightChangeIntent,
        ports::{
            LightConfigChanger as _,
            LightStateChanger as _,
            PersistentDataReader as _,
        },
    },
    infrastructure::{
        adapters::{bind_boot_button, BootButtonCallback},
        drivers::init_network_stack_ap,
        services::{
            LightStateService,
            PersistenceService,
            init_light_service,
            init_storage_services,
        },
        tasks::factory::{
            dhcp_server_task,
            factory_network_runner_task,
            factory_wifi_ap_task,
            http_server_task,
        },
    },
    mk_static,
};
use static_cell::StaticCell;

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

    // Initialize flash storage (shared between HTTP server for config and OTA)
    let flash = FLASH_STORAGE.init(FlashStorage::new(peripherals.FLASH));
    let flash_ptr = flash as *mut FlashStorage<'static>;

    // Initialize network stack for AP mode
    let (stack, runner, controller) = init_network_stack_ap(peripherals.WIFI);

    // Spawn WiFi AP task
    spawner.spawn(factory_wifi_ap_task(controller)).ok();

    // Spawn network runner
    spawner.spawn(factory_network_runner_task(runner)).ok();

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

    let mut light_service = init_light_service(
        spawner,
        peripherals.RMT,
        myrtio_esp_light::led_gpio!(peripherals),
    );

    let (ota_service, persistence_service) =
        init_storage_services(spawner, flash_ptr).await;

    let (_light_state, config) = persistence_service
        .read_persistent_data()
        .unwrap_or_default();

    light_service
        .set_config(config.light)
        .expect("Failed to set light config");

    let firmware = FirmwareUsecases::new(ota_service);

    let configuration = mk_static!(
        ConfigurationUsecases<PersistenceService, LightStateService>,
        ConfigurationUsecases::new(persistence_service.clone(), light_service.clone())
    );

    light_service
        .apply_light_intent(LightChangeIntent {
            power: Some(true),
            brightness: Some(255),
            color: Some((255, 255, 255)),
            color_temp: None,
            mode_id: Some(myrtio_light_composer::ModeId::Static as u8),
        })
        .unwrap();

    let handler = init_factory_controllers(configuration, firmware).await;

    bind_boot_button(
        peripherals.IO_MUX,
        peripherals.GPIO0,
        handle_boot_button_click,
    );

    // Spawn HTTP server
    spawner.spawn(http_server_task(stack, handler)).ok();

    println!("Factory firmware ready!");
    println!("Connect to WiFi: MyrtIO-Setup-XXXX");
    println!("Open http://192.168.4.1 in browser");

    let mut pin =
        Output::new(peripherals.GPIO2, Level::High, OutputConfig::default());
    loop {
        pin.set_high();
        embassy_time::Timer::after(Duration::from_millis(500)).await;
        pin.set_low();
        embassy_time::Timer::after(Duration::from_millis(500)).await;
    }
}
