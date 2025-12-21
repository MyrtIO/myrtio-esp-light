#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Duration;

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};
use esp_println::println;
use esp_storage::FlashStorage;
use static_cell::StaticCell;

use myrtio_esp_light::app::LightUsecases;
use myrtio_esp_light::config::{DeviceConfig, LIGHT_STATE_PARTITION_OFFSET};
use myrtio_esp_light::controllers::init_controllers;
use myrtio_esp_light::domain::entity::LightState;
use myrtio_esp_light::domain::ports::{OnBootHandler, PersistenceHandler};
use myrtio_esp_light::infrastructure::drivers::{init_network_stack, wait_for_connection};
use myrtio_esp_light::infrastructure::repositories::AppPersistentStorage;
use myrtio_esp_light::infrastructure::services::{
    LightStatePersistenceService, LightStateService, get_persistence_receiver,
};
use myrtio_esp_light::infrastructure::tasks::light_composer::{
    LightTaskParams, init_light_composer, light_composer_task,
};
use myrtio_esp_light::infrastructure::tasks::{
    mqtt_runtime_task, network_runner_task, persistence_task, wifi_connection_task,
};

esp_bootloader_esp_idf::esp_app_desc!();

static FLASH_STORAGE: StaticCell<FlashStorage<'static>> = StaticCell::new();

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

    // Start rtos
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    // Initialize flash storage and get initial state + config
    let flash = FLASH_STORAGE.init(FlashStorage::new(peripherals.FLASH));
    let flash_ptr = flash as *mut FlashStorage<'static>;
    let storage = AppPersistentStorage::new(flash_ptr, LIGHT_STATE_PARTITION_OFFSET);
    let persistent_data = storage.get_persistent_data();
    let initial_state: Option<LightState> =
        persistent_data.as_ref().map(|(_, state, _)| state.clone());
    let device_config: Option<DeviceConfig> = persistent_data.map(|(_, _, config)| config);

    // Spawn persistence task
    let receiver = get_persistence_receiver();
    spawner.spawn(persistence_task(storage, receiver)).ok();

    let persistence_service = LightStatePersistenceService::new();
    let light_config = device_config.as_ref().map(|cfg| cfg.light).unwrap_or(
        myrtio_esp_light::config::LightConfig {
            brightness_min: 0,
            brightness_max: 255,
            led_count: 20,
            skip_leds: 0,
            color_correction: 0xFFFFFF,
        },
    );

    // Initialize light composer and spawn its task
    let (driver, cmd_sender) =
        init_light_composer(peripherals.RMT, myrtio_esp_light::led_gpio!(peripherals));
    spawner
        .spawn(light_composer_task(
            driver,
            LightTaskParams {
                min_brightness: light_config.brightness_min,
                max_brightness: light_config.brightness_max,
                led_count: light_config.led_count,
                skip_leds: light_config.skip_leds,
                color_correction: light_config.color_correction,
            },
        ))
        .ok();

    // Initialize usecases and controllers
    let state_service = LightStateService::new(cmd_sender);
    let usecases = myrtio_esp_light::mk_static!(
        LightUsecases<LightStateService, LightStatePersistenceService>,
        LightUsecases::new(state_service, persistence_service)
    );
    let (mqtt_module, boot_controller) = init_controllers(usecases);
    boot_controller.on_boot(initial_state);

    // Validate config and start network if provisioned
    let config_valid = device_config
        .as_ref()
        .is_some_and(|cfg| !cfg.wifi.ssid.is_empty() && !cfg.mqtt.host.is_empty());

    if !config_valid {
        println!("app: no provisioned config; run factory provisioning");
        println!("app: wifi and mqtt will not start");
        loop {
            embassy_time::Timer::after(Duration::from_secs(60)).await;
        }
    }

    let device_config = device_config.unwrap();
    println!("app: using wifi ssid: {}", device_config.wifi.ssid);
    println!(
        "app: using mqtt host: {}:{}",
        device_config.mqtt.host, device_config.mqtt.port
    );

    // Initialize network stack and spawn network tasks
    let (stack, runner, controller) = init_network_stack(peripherals.WIFI);
    spawner
        .spawn(wifi_connection_task(controller, device_config.wifi.clone()))
        .ok();
    spawner.spawn(network_runner_task(runner)).ok();

    // Wait for network connection before starting network-dependent tasks
    wait_for_connection(stack).await;

    // Spawn MQTT task
    spawner
        .spawn(mqtt_runtime_task(
            stack,
            mqtt_module,
            device_config.mqtt.clone(),
        ))
        .ok();

    loop {
        embassy_time::Timer::after(Duration::from_secs(5)).await;
    }
}
