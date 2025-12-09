//! Static color fill effect
//!
//! Fills all LEDs with a single solid color.
//! Supports smooth color transitions via [`ColorTransition`].

use embassy_time::{Duration, Instant};

use super::Mode;
use crate::color::Rgb;
use crate::transition::ValueTransition;

/// Static color effect - fills all LEDs with one color
///
/// Supports smooth crossfade transitions when changing colors.
#[derive(Debug, Clone)]
pub struct StaticColorMode {
    /// Color with transition support
    color: ValueTransition<Rgb>,
}

impl StaticColorMode {
    /// Create a new static color effect
    pub fn new(color: Rgb) -> Self {
        Self {
            color: ValueTransition::new_rgb(color),
        }
    }

    /// Set the color with smooth transition
    ///
    /// # Arguments
    /// * `color` - Target color
    /// * `duration` - Transition duration
    pub fn set_color(&mut self, color: Rgb, duration: Duration, now: Instant) {
        self.color.set(color, duration, now);
    }
}

impl Mode for StaticColorMode {
    fn render<const N: usize>(&mut self, frame_time: Instant) -> [Rgb; N] {
        self.color.tick(frame_time);

        [self.color.current(); N]
    }

    fn is_transitioning(&self) -> bool {
        self.color.is_transitioning()
    }
}
