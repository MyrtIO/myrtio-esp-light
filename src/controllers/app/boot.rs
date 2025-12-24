use core::cell::RefCell;

use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use embassy_time::{Duration, Timer};
use embedded_hal::delay::DelayNs;
use esp_hal::{
    gpio::{Event, Input, InputConfig, Io, Pull},
    handler,
    peripherals::{GPIO0, IO_MUX},
    ram,
};
use esp_println::println;

use super::LIGHT_USECASES;
use crate::{
    domain::{
        dto::LightChangeIntent,
        ports::{BootManagerPort, OnBootHandler, PersistenceHandler},
    },
    infrastructure::repositories::{AppPersistentStorage, BootManager},
};

#[derive(Default)]
pub struct BootController<P: PersistenceHandler, B: BootManagerPort> {
    persistence: P,
    boot_manager: B,
}

impl<P: PersistenceHandler, B: BootManagerPort> BootController<P, B> {
    pub fn new(persistence: P, boot_manager: B) -> Self {
        Self {
            persistence,
            boot_manager,
        }
    }
}

impl<P: PersistenceHandler, B: BootManagerPort> OnBootHandler
    for BootController<P, B>
{
    fn on_boot_start(&mut self) {
        println!("[BOOT] system starting");
    }

    fn on_light_ready(&mut self) {
        println!("[BOOT] light is ready, applying stored state");

        let state = self
            .persistence
            .get_persistent_data()
            .map(|(_, state, _)| state)
            .unwrap_or_default();
        let intent: LightChangeIntent = state.into();

        LIGHT_USECASES.lock(|cell| {
            let mut cell_ref = cell.borrow_mut();
            cell_ref.as_mut().unwrap().apply_light_intent(intent).unwrap();
        });
    }

    fn on_boot_end(&mut self) {
        println!("Boot end");
    }
}

static BUTTON: Mutex<CriticalSectionRawMutex, RefCell<Option<Input>>> =
    Mutex::new(RefCell::new(None));

/// MQTT runtime task that accepts any module implementing `MqttModule`.
pub fn init_factory_reboot_button(
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

    // Timer::after(Duration::from_secs(3)).await;
    // boot_controller.on_magic_timeout();
}

#[handler]
#[ram]
fn handle_button_click() {
    BUTTON.lock(|cell: &RefCell<Option<Input<'static>>>| {
        let button = cell.borrow_mut().as_mut().unwrap();

        

        button.clear_interrupt();
    });
    
    // if critical_section::with(|cs| {
    //     BUTTON
    //         .borrow_ref_mut(cs)
    //         .as_mut()
    //         .unwrap()
    //         .is_interrupt_set()
    // }) {
    //     esp_println::println!("Button was the source of the interrupt");
    // } else {
    //     esp_println::println!("Button was not the source of the interrupt");
    // }

    critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt()
    });
}
