use core::cell::RefCell;

use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use esp_hal::{
    gpio::{Event, Input, InputConfig, InputPin, Io, Pull},
    handler,
    peripherals::IO_MUX,
    ram,
};

pub struct ButtonController;

/// Callback type for button click handler
pub type ButtonCallback = fn();

static BUTTON: Mutex<CriticalSectionRawMutex, RefCell<Option<Input>>> =
    Mutex::new(RefCell::new(None));

static CALLBACK: Mutex<CriticalSectionRawMutex, RefCell<Option<ButtonCallback>>> =
    Mutex::new(RefCell::new(None));

pub fn init_button_controller(
    mux: IO_MUX<'static>,
    pin: impl InputPin + 'static,
    on_click: ButtonCallback,
) -> ButtonController {
    let mut io = Io::new(mux);
    io.set_interrupt_handler(handle_button_click);
    let mut button = Input::new(pin, InputConfig::default().with_pull(Pull::Up));
    button.listen(Event::FallingEdge);

    BUTTON.lock(|cell| {
        cell.borrow_mut().replace(button);
    });

    CALLBACK.lock(|cell| {
        cell.borrow_mut().replace(on_click);
    });

    ButtonController
}

#[handler]
#[ram]
fn handle_button_click() {
    let is_button_interrupt = BUTTON.lock(|cell| {
        let mut cell = cell.borrow_mut();
        if let Some(button) = cell.as_mut() {
            let is_set = button.is_interrupt_set();
            button.clear_interrupt();
            is_set
        } else {
            false
        }
    });

    if is_button_interrupt {
        CALLBACK.lock(|cell| {
            if let Some(callback) = cell.borrow().as_ref() {
                callback();
            }
        });
    }
}
