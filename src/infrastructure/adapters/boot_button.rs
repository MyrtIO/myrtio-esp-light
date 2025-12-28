use core::cell::RefCell;

use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use esp_hal::{
    gpio::{Event, Input, InputConfig, InputPin, Io, Pull},
    handler,
    peripherals,
    ram,
};

/// Callback type for button click handler
pub type BootButtonCallback = fn();

/// Button input pin
static BUTTON: Mutex<CriticalSectionRawMutex, RefCell<Option<Input>>> =
    Mutex::new(RefCell::new(None));

/// Callback for button click handler
static CALLBACK: Mutex<
    CriticalSectionRawMutex,
    RefCell<Option<BootButtonCallback>>,
> = Mutex::new(RefCell::new(None));

/// Bind boot button to the system
pub fn bind_boot_button(
    mux: peripherals::IO_MUX<'static>,
    pin: impl InputPin + 'static,
    on_click: BootButtonCallback,
) {
    let mut io = Io::new(mux);
    io.set_interrupt_handler(handle_button_click);

    let config = InputConfig::default().with_pull(Pull::Up);
    let mut button = Input::new(pin, config);
    button.listen(Event::FallingEdge);

    BUTTON.lock(|cell| {
        cell.borrow_mut().replace(button);
    });
    CALLBACK.lock(|cell| {
        cell.borrow_mut().replace(on_click);
    });
}

/// Handler for boot button click event
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
