//! Static color fill effect
//!
//! Fills all LEDs with a single solid color.
//! Supports smooth color transitions via [`ColorTransition`].

use embassy_time::Duration;
use smart_leds::RGB;

use super::{EffectImpl};
use crate::transition::ColorTransition;

/// Default frame delta for transitions (~60fps)
const DEFAULT_FRAME_DELTA: Duration = Duration::from_millis(16);

/// Static color effect - fills all LEDs with one color
///
/// Supports smooth crossfade transitions when changing colors.
#[derive(Clone)]
pub struct StaticColorEffect {
    /// Color with transition support
    color: ColorTransition,
}

impl Default for StaticColorEffect {
    fn default() -> Self {
        Self {
            color: ColorTransition::new(RGB { r: 255, g: 255, b: 255 }),
        }
    }
}

impl StaticColorEffect {
    /// Create a new static color effect
    pub fn new(color: RGB<u8>) -> Self {
        Self {
            color: ColorTransition::new(color),
        }
    }

    /// Create from RGB values
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(RGB { r, g, b })
    }

    /// Set the color with smooth transition
    ///
    /// # Arguments
    /// * `color` - Target color
    /// * `duration` - Transition duration (0 for immediate)
    pub fn set_color(&mut self, color: RGB<u8>, duration: Duration) {
        self.color.set(color, duration);
    }

    /// Set the color immediately (no transition)
    pub fn set_color_immediate(&mut self, color: RGB<u8>) {
        self.color.set_immediate(color);
    }

    /// Get the current displayed color
    pub fn color(&self) -> RGB<u8> {
        self.color.current()
    }

    /// Check if a color transition is in progress
    pub fn is_transitioning(&self) -> bool {
        self.color.is_transitioning()
    }
}

impl<const N: usize> EffectImpl<N> for StaticColorEffect {
    fn render(&mut self, _time: Duration) -> [RGB<u8>; N] {
        // Update transition state
        self.color.tick(DEFAULT_FRAME_DELTA);
        
        [self.color.current(); N]
    }

    fn reset(&mut self) {
        // Keep current color, no reset needed
    }
}
