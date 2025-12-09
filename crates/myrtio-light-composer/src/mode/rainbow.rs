//! Rainbow cycling effects
//!
//! Provides two rainbow effect variants:
//! - `RainbowEffect`: Uses fixed-point HSV gradient algorithm (ported from `FastLED`)
//! - `RainbowFlowEffect`: Three-point mirrored gradient with smooth flow

use core::cmp::min;
use embassy_time::{Duration, Instant};

use super::Mode;
use crate::color::{fill_gradient_three_fp, mirror_half, Hsv, Rgb};

const DEFAULT_CYCLE_MS: u64 = 12_000;
const HUE_STEP: u8 = 60;

/// Rainbow effect using fixed-point HSV gradient algorithm
///
/// This implementation is ported from the `FastLED` `fillGradient` function
/// and uses 8.24 fixed-point arithmetic for smooth color transitions.
#[derive(Debug, Clone)]
pub struct RainbowMode {
    /// Duration of one complete rainbow cycle
    cycle_duration: Duration,
    /// Brightness value (0-255)
    value: u8,
    /// Saturation (0-255)
    saturation: u8,
}

impl Default for RainbowMode {
    fn default() -> Self {
        Self {
            cycle_duration: Duration::from_millis(DEFAULT_CYCLE_MS),
            value: 255,
            saturation: 255,
        }
    }
}

impl RainbowMode {
    /// Create a new rainbow mode with custom parameters
    pub fn new(cycle_duration: Duration, value: u8, saturation: u8) -> Self {
        Self {
            cycle_duration,
            value,
            saturation,
        }
    }

    /// Set the cycle duration
    #[must_use]
    pub fn with_cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration;
        self
    }

    /// Set the brightness value
    #[must_use]
    pub fn with_value(mut self, value: u8) -> Self {
        self.value = value;
        self
    }

    /// Set the saturation
    #[must_use]
    pub fn with_saturation(mut self, saturation: u8) -> Self {
        self.saturation = saturation;
        self
    }
}

impl Mode for RainbowMode {
    fn render<const N: usize>(&mut self, now: Instant) -> [Rgb; N] {
        let mut leds = [Rgb::default(); N];
        if N == 0 {
            return leds;
        }

        let cycle_ms = self.cycle_duration.as_millis().max(1);
        let progress_ms = now.as_millis() % cycle_ms;
        #[allow(clippy::cast_possible_truncation)]
        let base_hue = ((progress_ms * 255) / cycle_ms) as u8;

        let c1 = Hsv {
            hue: base_hue,
            sat: self.saturation,
            val: self.value,
        };
        let c2 = Hsv {
            hue: base_hue.wrapping_add(HUE_STEP),
            sat: self.saturation,
            val: self.value,
        };
        let c3 = Hsv {
            hue: base_hue.wrapping_add(HUE_STEP * 2),
            sat: self.saturation,
            val: self.value,
        };

        // Compute center for mirroring
        let mut center_len = N / 2;
        if !N.is_multiple_of(2) {
            center_len += 1;
        }
        center_len = min(center_len, N);

        // Fill first half with three-point gradient using fixed-point math
        {
            let (first_half, _) = leds.split_at_mut(center_len);
            fill_gradient_three_fp(first_half, c1, c2, c3);
        }

        // Mirror to second half
        mirror_half(&mut leds);
        leds
    }
}
