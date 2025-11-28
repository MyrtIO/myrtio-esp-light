//! Rainbow cycling effect
//!
//! Creates a smooth rainbow gradient that cycles through all hues.

use embassy_time::Duration;
use smart_leds::{
    RGB,
    hsv::{Hsv, hsv2rgb},
};

use super::{EffectImpl};

/// Rainbow effect configuration and state
#[derive(Clone)]
pub struct RainbowEffect {
    /// Duration of one complete rainbow cycle
    cycle_duration: Duration,
    /// Brightness value (0-255)
    value: u8,
    /// Saturation (0-255)
    saturation: u8,
}

impl Default for RainbowEffect {
    fn default() -> Self {
        Self {
            cycle_duration: Duration::from_millis(3000),
            value: 255,
            saturation: 255,
        }
    }
}

impl RainbowEffect {
    /// Create a new rainbow effect with custom parameters
    pub fn new(cycle_duration: Duration, value: u8, saturation: u8) -> Self {
        Self {
            cycle_duration,
            value,
            saturation,
        }
    }

    /// Set the cycle duration
    pub fn with_cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration;
        self
    }

    /// Set the brightness value
    pub fn with_value(mut self, value: u8) -> Self {
        self.value = value;
        self
    }

    /// Set the saturation
    pub fn with_saturation(mut self, saturation: u8) -> Self {
        self.saturation = saturation;
        self
    }
}

impl<const N: usize> EffectImpl<N> for RainbowEffect {
    fn render(&mut self, time: Duration) -> [RGB<u8>; N] {
        let cycle_ms = self.cycle_duration.as_millis() as u64;
        let progress_ms = time.as_millis() % cycle_ms;

        // Calculate base hue from time progress (0-255)
        let base_hue = ((progress_ms as f32 / cycle_ms as f32) * 255.0) as u8;

        core::array::from_fn(|i| {
            // Offset hue for each LED to create gradient
            let led_offset = (i as u16 * 255 / N as u16) as u8;
            let hue = base_hue.wrapping_add(led_offset);

            hsv2rgb(Hsv {
                hue,
                sat: self.saturation,
                val: self.value,
            })
        })
    }

    fn reset(&mut self) {
        // Rainbow effect is stateless relative to start time,
        // so no reset needed
    }
}
