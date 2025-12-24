#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Duration;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};
use esp_println::println;
use esp_storage::FlashStorage;
use myrtio_esp_light::{
    app::{ConfigurationUsecases, FirmwareUsecases, LightUsecases},
    controllers::init_app_controllers,
    domain::ports::{LightConfigChanger, LightStateChanger, PersistentDataReader},
    infrastructure::{
        drivers::{init_network_stack, wait_for_connection},
        services::{
            LightStateService,
            PersistenceService,
            init_light_service,
            init_storage_services,
        },
        tasks::app::{mqtt_client_task, network_runner_task, wifi_connection_task},
    },
    mk_static,
};
use static_cell::StaticCell;

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

    let (ota_service, persistence_service) =
        init_storage_services(spawner, flash_ptr).await;

    let mut light_service = init_light_service(
        spawner,
        peripherals.RMT,
        myrtio_esp_light::led_gpio!(peripherals),
    );

    let (light_state, config) = persistence_service
        .read_persistent_data()
        .unwrap_or_default();

    light_service
        .apply_light_intent(light_state.into())
        .unwrap();
    light_service.set_config(config.light).unwrap();

    let configuration = mk_static!(
        ConfigurationUsecases<PersistenceService, LightStateService>,
        ConfigurationUsecases::new(persistence_service.clone(), light_service.clone())
    );
    let light = mk_static!(
        LightUsecases<LightStateService, PersistenceService>,
        LightUsecases::new(light_service, persistence_service)
    );

    let firmware = FirmwareUsecases::new(ota_service);

    let mqtt_module = init_app_controllers(light);

    // Validate config and start network if provisioned
    let config_valid = !config.wifi.ssid.is_empty() && !config.mqtt.host.is_empty();

    if !config_valid {
        println!("app: no provisioned config; run factory provisioning");
        println!("app: wifi and mqtt will not start");
        loop {
            embassy_time::Timer::after(Duration::from_secs(60)).await;
        }
    }

    println!("app: using wifi ssid: {}", config.wifi.ssid);
    println!(
        "app: using mqtt host: {}:{}",
        config.mqtt.host, config.mqtt.port
    );

    // Initialize network stack and spawn network tasks
    let (stack, runner, controller) = init_network_stack(peripherals.WIFI);
    spawner
        .spawn(wifi_connection_task(controller, config.wifi.clone()))
        .ok();
    spawner.spawn(network_runner_task(runner)).ok();

    // Wait for network connection before starting network-dependent tasks
    wait_for_connection(stack).await;

    // Spawn MQTT task
    spawner
        .spawn(mqtt_client_task(stack, mqtt_module, config.mqtt.clone()))
        .ok();

    loop {
        embassy_time::Timer::after(Duration::from_secs(5)).await;
    }
}
