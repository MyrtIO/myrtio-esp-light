use esp_hal::xtensa_lx::interrupt;
use esp_hal::{gpio::interconnect::PeripheralOutput, peripherals::RMT, rmt::Rmt, time::Rate};
use esp_hal_smartled::{SmartLedsAdapter, buffer_size, smart_led_buffer};
use smart_leds::{RGB, SmartLedsWrite};
use static_cell::make_static;

use myrtio_light_composer::LedDriver;

use crate::infrastructure::config;

/// ESP-specific LED driver using RMT peripheral
///
/// This driver uses the ESP32's RMT (Remote Control) peripheral
/// to generate the precise timing signals required by WS2812B LEDs.
pub(crate) struct EspLedDriver<'a, const N: usize> {
    adapter: SmartLedsAdapter<'a, { buffer_size(config::LIGHT_LED_COUNT) }>,
}

impl<'a, const N: usize> EspLedDriver<'a, N> {
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
        let rmt_buffer = make_static!(smart_led_buffer!(config::LIGHT_LED_COUNT));
        let adapter = SmartLedsAdapter::new(rmt.channel0, pin, rmt_buffer);

        Self { adapter }
    }
}

impl<const N: usize> LedDriver<N> for EspLedDriver<'static, N> {
    fn write(&mut self, colors: &[RGB<u8>; N]) {
        interrupt::free(|| {
            let _ = self.adapter.write(colors.iter().copied());
        });
    }
}
