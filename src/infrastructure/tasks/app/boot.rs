use core::cell::RefCell;

use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Event, Input, InputConfig, Io, Pull},
    handler,
    peripherals::{self, GPIO0, IO_MUX},
    ram,
};

use crate::{
    controllers::BootController,
    domain::ports::OnBootHandler,
    infrastructure::repositories::{AppPersistentStorage, BootManager},
};

static BUTTON: Mutex<CriticalSectionRawMutex, RefCell<Option<Input>>> =
    Mutex::new(RefCell::new(None));

static BOOT_MANAGER_CHANNEL: Channel<CriticalSectionRawMutex, BootManagerCommand> = Channel::new();

pub async fn boot_manager_task(
    mut boot_controller: BootController<AppPersistentStorage, BootManager>,
) {


    loop {
        embassy_time::Timer::after(Duration::from_secs(1)).await;
    }
}

/// MQTT runtime task that accepts any module implementing `MqttModule`.
pub async fn init_factory_reboot_button(
    mut boot_controller: BootController<AppPersistentStorage, BootManager>,
    mux: IO_MUX<'static>,
    gpio: GPIO0<'static>,
) {
    let mut io = Io::new(mux);
    io.set_interrupt_handler(handle_button_click);
    let config = InputConfig::default().with_pull(Pull::Up);
    let mut button = Input::new(gpio, config);
    button.listen(Event::FallingEdge);

    BUTTON.lock(|cell| {
        cell.borrow_mut().replace(button);
    });

    Timer::after(Duration::from_secs(3)).await;
    boot_controller.on_magic_timeout();
}

#[handler]
#[ram]
fn handle_button_click() {
    esp_println::println!(
        "GPIO Interrupt with priority {}",
        esp_hal::xtensa_lx::interrupt::get_level()
    );

    let button = BUTTON.lock(|cell| cell.borrow_mut().as_mut().unwrap());

    if critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .is_interrupt_set()
    }) {
        esp_println::println!("Button was the source of the interrupt");
    } else {
        esp_println::println!("Button was not the source of the interrupt");
    }

    critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt()
    });
}
