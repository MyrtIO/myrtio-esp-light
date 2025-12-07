//! Color transition utility for smooth color changes
//!
//! Provides a reusable component for animating between colors
//! with configurable duration.

use embassy_time::Duration;
use smart_leds::RGB;

use crate::math8::{blend8, progress8};

/// Color transition with smooth blending
#[derive(Clone)]
pub struct ColorTransition {
    /// Current interpolated color
    current: RGB<u8>,
    /// Color at the start of transition
    from: RGB<u8>,
    /// Target color (None if no transition in progress)
    target: Option<RGB<u8>>,
    /// Total transition duration
    duration: Duration,
    /// Time elapsed since transition start
    elapsed: Duration,
}

impl Default for ColorTransition {
    fn default() -> Self {
        Self::new(RGB::default())
    }
}

impl ColorTransition {
    /// Create a new color transition with initial color
    pub fn new(initial: RGB<u8>) -> Self {
        Self {
            current: initial,
            from: initial,
            target: None,
            duration: Duration::from_millis(0),
            elapsed: Duration::from_millis(0),
        }
    }

    /// Get current color value
    pub fn current(&self) -> RGB<u8> {
        self.current
    }

    /// Check if a transition is in progress
    pub fn is_transitioning(&self) -> bool {
        self.target.is_some()
    }

    /// Set color immediately (no transition)
    pub fn set_immediate(&mut self, color: RGB<u8>) {
        self.current = color;
        self.from = color;
        self.target = None;
        self.elapsed = Duration::from_millis(0);
    }

    /// Set color with smooth transition
    pub fn set(&mut self, color: RGB<u8>, duration: Duration) {
        if duration.as_millis() == 0 {
            self.set_immediate(color);
            return;
        }

        self.from = self.current;
        self.target = Some(color);
        self.duration = duration;
        self.elapsed = Duration::from_millis(0);
    }

    /// Update transition state
    ///
    /// Call this once per frame with the frame delta time.
    pub fn tick(&mut self, delta: Duration) {
        if let Some(target) = self.target {
            self.elapsed += delta;

            if self.elapsed >= self.duration {
                // Transition complete
                self.current = target;
                self.from = target;
                self.target = None;
            } else {
                let progress = progress8(self.elapsed, self.duration);
                self.current = blend_colors(self.from, target, progress);
            }
        }
    }
}

/// Blend two RGB colors
///
/// # Arguments
/// * `a` - First color
/// * `b` - Second color
/// * `amount_of_b` - Blend factor (0 = all a, 255 = all b)
#[inline]
pub fn blend_colors(a: RGB<u8>, b: RGB<u8>, amount_of_b: u8) -> RGB<u8> {
    RGB {
        r: blend8(a.r, b.r, amount_of_b),
        g: blend8(a.g, b.g, amount_of_b),
        b: blend8(a.b, b.b, amount_of_b),
    }
}
