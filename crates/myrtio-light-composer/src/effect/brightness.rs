//! Brightness envelope for smooth fades
//!
//! Handles global brightness control with smooth transitions.
//! Used for:
//! - Global brightness setting
//! - Fade-in when turning on
//! - Fade-out when turning off
//! - Fade-out-in during effect changes

use embassy_time::{Duration, Instant};

#[cfg(feature = "log")]
use esp_println::println;

use super::Effect;
use crate::{
    color::Rgb,
    math8::{U8Adjuster, scale8},
    transition::ValueTransition,
};

/// Configuration for the brightness effect
#[derive(Debug, Clone)]
pub struct BrightnessEffectConfig {
    /// Minimum brightness
    pub min_brightness: u8,
    /// Scale factor (0-255 = 0.0-1.0)
    pub scale: u8,
    /// Adjustment function
    pub adjust: Option<U8Adjuster>,
}

/// Brightness transition and correction
#[derive(Debug, Clone)]
pub(crate) struct BrightnessEffect {
    /// Scale factor (0-255 = 0.0-1.0)
    min_brightness: u8,
    scale: u8,
    adjust: Option<U8Adjuster>,
    /// Current brightness value (0-255)
    brightness: ValueTransition<u8>,
}

impl BrightnessEffect {
    /// Create a new brightness effect
    pub(crate) const fn new(brightness: u8, config: &BrightnessEffectConfig) -> Self {
        Self {
            min_brightness: config.min_brightness,
            scale: config.scale,
            adjust: config.adjust,
            brightness: ValueTransition::new_u8(brightness),
        }
    }

    /// Set brightness with smooth transition
    pub(crate) fn set(&mut self, brightness: u8, duration: Duration, now: Instant) {
        let brightness = brightness.saturating_sub(self.min_brightness);
        let corrected_brightness =
            scale8(brightness, self.scale).saturating_add(self.min_brightness);
        #[cfg(feature = "log")]
        println!(
            "[BrightnessEffect.set] setting brightness to {:?} ({:?})",
            brightness, corrected_brightness
        );
        self.brightness.set(corrected_brightness, duration, now);
    }

    pub(crate) fn set_uncorrected(&mut self, brightness: u8, duration: Duration, now: Instant) {
        self.brightness.set(brightness, duration, now);
    }

    /// Check if a transition is in progress
    pub(crate) const fn is_transitioning(&self) -> bool {
        self.brightness.is_transitioning()
    }
}

impl Effect for BrightnessEffect {
    fn apply<const N: usize>(&mut self, frame: &mut [Rgb; N]) {
        let mut current = self.brightness.current();

        if current == 255 {
            return;
        }

        if current == 0 {
            for pixel in frame.iter_mut() {
                *pixel = Rgb { r: 0, g: 0, b: 0 };
            }
            return;
        }

        if let Some(adjust) = self.adjust {
            current = adjust(current);
        }

        for pixel in frame.iter_mut() {
            pixel.r = scale8(pixel.r, current);
            pixel.g = scale8(pixel.g, current);
            pixel.b = scale8(pixel.b, current);
        }
    }

    fn tick(&mut self, now: Instant) {
        self.brightness.tick(now);
    }
}
