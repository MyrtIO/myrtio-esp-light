//! Effect system with compile-time known effect variants
//!
//! All effects are stored in an enum to avoid heap allocations.
//! Each effect implements the `EffectImpl` trait.

mod rainbow;
mod static_color;

pub use rainbow::RainbowEffect;
pub use static_color::StaticColorEffect;

use crate::state::EffectId;
use embassy_time::Duration;
use smart_leds::RGB;

/// Trait for effect implementations
///
/// Each effect must be able to render a frame given the current time.
/// Effects are stateful and can maintain internal state between frames.
pub trait EffectImpl<const N: usize> {
    /// Render a single frame
    ///
    /// # Arguments
    /// * `time` - Current time since system start (for animations)
    ///
    /// # Returns
    /// Array of RGB colors for each LED
    fn render(&mut self, time: Duration) -> [RGB<u8>; N];

    /// Reset effect state
    ///
    /// Called when effect is activated after being inactive.
    /// Use this to reset animation progress, timers, etc.
    fn reset(&mut self) {}
}

/// Effect slot - enum containing all possible effects
///
/// Using an enum instead of trait objects allows:
/// - Zero heap allocations
/// - Known size at compile time
/// - Better optimization opportunities
#[derive(Clone)]
pub enum EffectSlot<const N: usize> {
    /// No effect - all LEDs off
    Off,
    /// Rainbow cycling effect
    Rainbow(RainbowEffect),
    /// Static single color
    Static(StaticColorEffect),
}

impl<const N: usize> Default for EffectSlot<N> {
    fn default() -> Self {
        Self::Off
    }
}

impl<const N: usize> EffectSlot<N> {
    /// Render the current effect
    pub fn render(&mut self, time: Duration) -> [RGB<u8>; N] {
        match self {
            Self::Off => [RGB::default(); N],
            Self::Rainbow(effect) => effect.render(time),
            Self::Static(effect) => effect.render(time),
        }
    }

    /// Reset the effect state
    pub fn reset(&mut self) {
        match self {
            Self::Off => {}
            Self::Rainbow(effect) => EffectImpl::<N>::reset(effect),
            Self::Static(effect) => EffectImpl::<N>::reset(effect),
        }
    }

    /// Check if effect is Off
    pub fn is_off(&self) -> bool {
        matches!(self, Self::Off)
    }

    /// Get the effect ID for external observation
    pub fn effect_id(&self) -> EffectId {
        match self {
            Self::Off => EffectId::Off,
            Self::Rainbow(_) => EffectId::Rainbow,
            Self::Static(_) => EffectId::Static,
        }
    }

    /// Get the current color (for static effect)
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Self::Static(effect) => {
                let c = effect.color();
                (c.r, c.g, c.b)
            }
            _ => (255, 255, 255),
        }
    }

    /// Update the color of the current effect with optional transition.
    ///
    /// No-ops when the active effect does not support color updates.
    pub fn set_color(&mut self, color: RGB<u8>, duration: Duration) {
        if let Self::Static(effect) = self {
            effect.set_color(color, duration);
        }
    }

    /// Convenience wrapper for setting the effect color from RGB values.
    pub fn set_color_rgb(&mut self, r: u8, g: u8, b: u8, duration: Duration) {
        self.set_color(RGB { r, g, b }, duration);
    }
}
