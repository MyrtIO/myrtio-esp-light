//! Effect system with compile-time known effect variants
//!
//! All effects are stored in an enum to avoid heap allocations.
//! Each effect implements the `EffectImpl` trait.

mod rainbow;
mod static_color;

pub use rainbow::{RainbowEffect, RainbowFlowEffect};
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
    /// Rainbow cycling effect (fixed-point gradient)
    Rainbow(RainbowEffect),
    /// Rainbow flow effect (three-point mirrored gradient)
    RainbowFlow(RainbowFlowEffect),
    /// Static single color
    Static(StaticColorEffect),
}

impl<const N: usize> Default for EffectSlot<N> {
    fn default() -> Self {
        Self::Off
    }
}

/// Known effect names that can be requested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectName {
    /// Static single color
    Static,
    /// Rainbow cycling effect
    Rainbow,
    /// Rainbow flow effect (three-point mirrored)
    RainbowFlow,
}

pub const EFFECT_NAME_STATIC: &str = "static";
pub const EFFECT_NAME_RAINBOW: &str = "rainbow";
pub const EFFECT_NAME_RAINBOW_FLOW: &str = "rainbow_flow";

impl EffectName {
    pub fn to_effect_slot<const N: usize>(self, r: u8, g: u8, b: u8) -> EffectSlot<N> {
        match self {
            Self::Static => EffectSlot::Static(StaticColorEffect::from_rgb(r, g, b)),
            Self::Rainbow => EffectSlot::Rainbow(RainbowEffect::default()),
            Self::RainbowFlow => EffectSlot::RainbowFlow(RainbowFlowEffect::default()),
        }
    }

    pub const fn from_id(id: EffectId) -> Option<Self> {
        Some(match id {
            EffectId::Static => Self::Static,
            EffectId::Rainbow => Self::Rainbow,
            EffectId::RainbowFlow => Self::RainbowFlow,
            EffectId::Off => return None,
        })
    }

    pub const fn to_id(self) -> Option<EffectId> {
        Some(match self {
            Self::Static => EffectId::Static,
            Self::Rainbow => EffectId::Rainbow,
            Self::RainbowFlow => EffectId::RainbowFlow,
        })
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Static => EFFECT_NAME_STATIC,
            Self::Rainbow => EFFECT_NAME_RAINBOW,
            Self::RainbowFlow => EFFECT_NAME_RAINBOW_FLOW,
        }
    }

    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s {
            EFFECT_NAME_STATIC => Some(Self::Static),
            EFFECT_NAME_RAINBOW => Some(Self::Rainbow),
            EFFECT_NAME_RAINBOW_FLOW => Some(Self::RainbowFlow),
            _ => None,
        }
    }
}

impl<const N: usize> EffectSlot<N> {
    /// Render the current effect
    pub fn render(&mut self, time: Duration) -> [RGB<u8>; N] {
        match self {
            Self::Off => [RGB::default(); N],
            Self::Rainbow(effect) => effect.render(time),
            Self::RainbowFlow(effect) => effect.render(time),
            Self::Static(effect) => effect.render(time),
        }
    }

    /// Reset the effect state
    pub fn reset(&mut self) {
        match self {
            Self::Off => {}
            Self::Rainbow(effect) => EffectImpl::<N>::reset(effect),
            Self::RainbowFlow(effect) => EffectImpl::<N>::reset(effect),
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
            Self::Rainbow(_) | Self::RainbowFlow(_) => EffectId::Rainbow,
            Self::Static(_) => EffectId::Static,
        }
    }

    /// Get the current color if the effect supports colors.
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Self::Static(effect) => {
                let c = effect.color();
                (c.r, c.g, c.b)
            }
            _ => (255, 255, 255),
        }
    }

    /// Return the current color only when the effect actually exposes one.
    pub fn color_if_supported(&self) -> Option<(u8, u8, u8)> {
        match self {
            Self::Static(_) => Some(self.color()),
            _ => None,
        }
    }

    /// Update the color of the current effect with optional transition.
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
