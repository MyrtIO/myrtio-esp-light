//! Brightness envelope for smooth fades
//!
//! Handles global brightness control with smooth transitions.
//! Used for:
//! - Global brightness setting
//! - Fade-in when turning on
//! - Fade-out when turning off
//! - Fade-out-in during effect changes

use embassy_time::Duration;
use smart_leds::RGB;

/// Brightness envelope with smooth transitions
#[derive(Clone)]
pub struct BrightnessEnvelope<const N: usize> {
    /// Scale factor (0-255 = 0.0-1.0)
    scale: u8,
    /// Current brightness value (0-255)
    current: u8,
    /// Target brightness for transition
    target: u8,
    /// Transition state
    transition: Option<BrightnessTransition>,
    /// Frame time
    frame_time: Duration,
}

#[derive(Clone)]
struct BrightnessTransition {
    /// Starting brightness
    start_value: u8,
    /// Target brightness
    end_value: u8,
    /// Total transition duration
    duration: Duration,
    /// Time elapsed since transition start
    elapsed: Duration,
}

impl<const N: usize> Default for BrightnessEnvelope<N> {
    fn default() -> Self {
        Self {
            scale: 255,
            current: 255,
            target: 255,
            transition: None,
            frame_time: Duration::from_millis(16), // ~60fps default
        }
    }
}

impl<const N: usize> BrightnessEnvelope<N> {
    /// Create a new brightness envelope with initial brightness
    pub fn new(brightness: u8, frame_time: Duration) -> Self {
        Self {
            scale: 255,
            current: brightness,
            target: brightness,
            transition: None,
            frame_time,
        }
    }

    pub fn set_scale(&mut self, scale: u8) {
        self.scale = scale;
        self.current = scale8(self.current, scale);
        self.target = scale8(self.target, scale);
    }

    /// Set brightness immediately (no transition)
    pub fn set_immediate(&mut self, brightness: u8) {
        let scaled = self.scale(brightness);
        self.current = scaled;
        self.target = scaled;
        self.transition = None;
    }

    /// Set brightness with smooth transition
    pub fn set(&mut self, brightness: u8, duration: Duration) {
        if duration.as_millis() == 0 {
            self.set_immediate(brightness);
            return;
        }

        self.target = self.scale(brightness);
        self.transition = Some(BrightnessTransition {
            start_value: self.current,
            end_value: self.target,
            duration,
            elapsed: Duration::from_millis(0),
        });
    }

    /// Fade out to zero
    pub fn fade_out(&mut self, duration: Duration) {
        self.set(0, duration);
    }

    /// Fade in to target brightness
    pub fn fade_in(&mut self, target: u8, duration: Duration) {
        self.set(target, duration);
    }

    /// Check if a transition is in progress
    pub fn is_transitioning(&self) -> bool {
        self.transition.is_some()
    }

    /// Check if brightness is zero (fully faded out)
    pub fn is_faded_out(&self) -> bool {
        self.current == 0 && self.transition.is_none()
    }

    /// Update with specific delta time
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_lossless,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn tick(&mut self) {
        if let Some(ref mut transition) = self.transition {
            transition.elapsed += self.frame_time;

            if transition.elapsed >= transition.duration {
                // Transition complete
                self.current = transition.end_value;
                self.transition = None;
            } else {
                // Calculate progress (0.0 - 1.0)
                let progress =
                    transition.elapsed.as_millis() as f32 / transition.duration.as_millis() as f32;

                // Linear interpolation
                let start = transition.start_value as f32;
                let end = transition.end_value as f32;
                self.current = (start + (end - start) * progress) as u8;
            }
        }
    }

    /// Apply brightness to a frame
    pub fn apply(&self, frame: &mut [RGB<u8>; N]) {
        if self.current == 255 {
            // No scaling needed
            return;
        }

        if self.current == 0 {
            // Full black
            for pixel in frame.iter_mut() {
                *pixel = RGB::default();
            }
            return;
        }

        // Scale each pixel
        for pixel in frame.iter_mut() {
            pixel.r = scale8(pixel.r, self.current);
            pixel.g = scale8(pixel.g, self.current);
            pixel.b = scale8(pixel.b, self.current);
        }
    }

    fn scale(&self, value: u8) -> u8 {
        scale8(value, self.scale)
    }
}

/// Scale an 8-bit value by a factor (0-255 = 0.0-1.0)
///
/// Uses integer math for efficiency on embedded systems.
#[inline]
#[allow(clippy::cast_lossless)]
fn scale8(value: u8, scale: u8) -> u8 {
    ((value as u16 * scale as u16) >> 8) as u8
}
