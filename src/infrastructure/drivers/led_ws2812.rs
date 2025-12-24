use esp_hal::{
    gpio::interconnect::PeripheralOutput,
    peripherals::RMT,
    rmt::Rmt,
    time::Rate,
    xtensa_lx::interrupt,
};
use esp_hal_smartled::{SmartLedsAdapter, buffer_size, smart_led_buffer};
use myrtio_light_composer::{LedDriver, Rgb};
use smart_leds::SmartLedsWrite;
use static_cell::make_static;

pub(crate) const MAX_LED_COUNT: usize = 128;

/// ESP-specific LED driver using RMT peripheral
///
/// This driver uses the ESP32's RMT (Remote Control) peripheral
/// to generate the precise timing signals required by WS2812B LEDs.
pub struct EspLedDriver<'a> {
    adapter: SmartLedsAdapter<'a, { buffer_size(MAX_LED_COUNT) }>,
}

impl<'a> EspLedDriver<'a> {
    /// Create a new ESP LED driver
    ///
    /// # Arguments
    /// * `rmt` - RMT peripheral
    /// * `pin` - GPIO pin connected to the LED data line
    pub(crate) fn new<O>(rmt: RMT<'a>, pin: O) -> Self
    where
        O: PeripheralOutput<'a>,
    {
        let rmt = Rmt::new(rmt, Rate::from_mhz(80)).unwrap();

        // Safety: This is a static buffer that lives for the entire program
        // We use make_static! to ensure the buffer has 'static lifetime
        let rmt_buffer = make_static!(smart_led_buffer!(MAX_LED_COUNT));
        let adapter = SmartLedsAdapter::new(rmt.channel0, pin, rmt_buffer);

        Self { adapter }
    }
}

impl LedDriver for EspLedDriver<'static> {
    fn write(&mut self, colors: &[Rgb]) {
        interrupt::free(|| {
            let _ = self.adapter.write(colors.iter().copied());
        });
    }
}
