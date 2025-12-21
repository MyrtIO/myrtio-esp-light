#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Duration;

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};
use esp_storage::FlashStorage;
use static_cell::StaticCell;

use myrtio_esp_light::app::LightUsecases;
use myrtio_esp_light::config::LIGHT_STATE_PARTITION_OFFSET;
use myrtio_esp_light::controllers::init_controllers;
use myrtio_esp_light::domain::ports::{OnBootHandler, PersistenceHandler};
use myrtio_esp_light::infrastructure::drivers::{init_network_stack, wait_for_connection};
use myrtio_esp_light::infrastructure::repositories::AppPersistentStorage;
use myrtio_esp_light::infrastructure::services::{
    LightStatePersistenceService, LightStateService, get_persistence_receiver,
};
use myrtio_esp_light::infrastructure::tasks::light_composer::{
    init_light_composer, light_composer_task,
};
use myrtio_esp_light::infrastructure::tasks::{
    mqtt_runtime_task, network_runner_task, persistence_task, wifi_connection_task,
};

esp_bootloader_esp_idf::esp_app_desc!();

static FLASH_STORAGE: StaticCell<FlashStorage<'static>> = StaticCell::new();

// static_cell::make_static! in main causes a compiler error
macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

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

    // Initialize flash storage and get initial state
    let flash = FLASH_STORAGE.init(FlashStorage::new(peripherals.FLASH));
    let flash_ptr = flash as *mut FlashStorage<'static>;
    let storage = AppPersistentStorage::new(flash_ptr, LIGHT_STATE_PARTITION_OFFSET);
    let initial_state = storage.get_persistent_data().map(|(_, state, _)| state);

    // Spawn persistence task
    let receiver = get_persistence_receiver();
    spawner.spawn(persistence_task(storage, receiver)).ok();

    let persistence_service = LightStatePersistenceService::new();

    // Initialize light composer and spawn its task
    let (driver, cmd_sender) =
        init_light_composer(peripherals.RMT, myrtio_esp_light::led_gpio!(peripherals));
    spawner.spawn(light_composer_task(driver)).ok();

    // Initialize usecases and controllers
    let state_service = LightStateService::new(cmd_sender);
    let usecases = mk_static!(
        LightUsecases<LightStateService, LightStatePersistenceService>,
        LightUsecases::new(state_service, persistence_service)
    );
    let (mqtt_module, boot_controller) = init_controllers(usecases);
    boot_controller.on_boot(initial_state);

    // Initialize network stack and spawn network tasks
    let (stack, runner, controller) = init_network_stack(peripherals.WIFI);
    spawner.spawn(wifi_connection_task(controller)).ok();
    spawner.spawn(network_runner_task(runner)).ok();

    // Wait for network connection before starting network-dependent tasks
    wait_for_connection(stack).await;

    // Spawn MQTT task
    spawner.spawn(mqtt_runtime_task(stack, mqtt_module)).ok();

    loop {
        embassy_time::Timer::after(Duration::from_secs(5)).await;
    }
}
