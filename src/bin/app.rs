#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};
use esp_println::println;
use myrtio_esp_light::{
    app::{FirmwareUsecases, LightUsecases},
    config::hostname,
    controllers::app::{handle_boot_button_click, init_app_controllers},
    domain::ports::{
        BootSectorSelector,
        LightConfigChanger,
        LightStateChanger,
        PersistentDataReader,
    },
    infrastructure::{
        adapters::{self},
        drivers::{self},
        services::{self},
        types::LightUsecasesImpl,
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

    // Start rtos
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    // Initialize flash services
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
    let (light_state, config) = persistence_service
        .read_persistent_data()
        .unwrap_or_default();
    light_service
        .apply_light_intent(light_state.into())
        .unwrap();
    light_service.set_config(config.light).unwrap();

    // Initialize usecases
    let light_usecases = mk_static!(
        LightUsecasesImpl,
        LightUsecases::new(light_service, persistence_service)
    );
    let mut firmware_usecases = FirmwareUsecases::new(firmware_service);

    // Validate config and start network if provisioned
    let config_valid = !config.wifi.ssid.is_empty() && !config.mqtt.host.is_empty();
    if !config_valid {
        println!("app: no provisioned config; rebooting to factory firmware");
        firmware_usecases.boot_factory().unwrap();
        loop {
            Timer::after(Duration::from_secs(60)).await;
        }
    }
    println!("app: using wifi ssid: {}", config.wifi.ssid);
    println!(
        "app: using mqtt host: {}:{}",
        config.mqtt.host, config.mqtt.port
    );
    let stack = drivers::start_wifi_sta(
        spawner,
        peripherals.WIFI,
        config.wifi.ssid,
        config.wifi.password,
        hostname(),
    )
    .await;

    // Initialize adapters
    let mqtt_module = init_app_controllers(light_usecases, firmware_usecases);
    adapters::bind_boot_button(
        peripherals.IO_MUX,
        peripherals.GPIO0,
        handle_boot_button_click,
    );
    adapters::start_mqtt_client(spawner, stack, mqtt_module, config.mqtt);

    loop {
        Timer::after(Duration::from_secs(5)).await;
    }
}
