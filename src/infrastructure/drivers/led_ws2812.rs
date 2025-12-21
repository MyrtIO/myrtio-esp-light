use static_cell::make_static;

use esp_hal::xtensa_lx::interrupt;
use esp_hal::{gpio::interconnect::PeripheralOutput, peripherals::RMT, rmt::Rmt, time::Rate};
use esp_hal_smartled::{SmartLedsAdapter, buffer_size, smart_led_buffer};
use smart_leds::SmartLedsWrite;

use myrtio_light_composer::{LedDriver, Rgb};

use crate::infrastructure::config;

pub(crate) const MAX_LED_COUNT: usize = 128;

/// ESP-specific LED driver using RMT peripheral
///
/// This driver uses the ESP32's RMT (Remote Control) peripheral
/// to generate the precise timing signals required by WS2812B LEDs.
pub struct EspLedDriver<'a> {
    adapter: SmartLedsAdapter<'a, { buffer_size(MAX_LED_COUNT) }>,
    skip_leds: usize,
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

        Self {
            adapter,
            skip_leds: 0,
        }
    }

    pub(crate) fn new_with_skip<O>(rmt: RMT<'a>, pin: O, skip_leds: usize) -> Self
    where
        O: PeripheralOutput<'a>,
    {
        let mut driver = Self::new(rmt, pin);
        driver.skip_leds = skip_leds;
        driver
    }
}

impl LedDriver for EspLedDriver<'static> {
    fn write<const N: usize>(&mut self, colors: &[Rgb; N]) {
        if config::LIGHT.skip_leds != 0 {
            let mut colors_with_skip: [Rgb; N] = [Rgb::new(0, 0, 0); N];
            for i in config::LIGHT.skip_leds..N {
                colors_with_skip[i] = colors[i - config::LIGHT.skip_leds];
            }
            interrupt::free(|| {
                let _ = self.adapter.write(colors_with_skip.iter().copied());
            });
        } else {
            interrupt::free(|| {
                let _ = self.adapter.write(colors.iter().copied());
            });
        }
    }
}
