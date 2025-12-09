//! Mode system with compile-time known mode variants
//!
//! All modes are stored in an enum to avoid heap allocations.
//! Each mode implements the `Mode` trait.

mod rainbow;
mod static_color;

use crate::color::Rgb;
use embassy_time::{Duration, Instant};

pub use rainbow::RainbowMode;
pub use static_color::StaticColorMode;

const MODE_NAME_STATIC: &str = "static";
const MODE_NAME_RAINBOW: &str = "rainbow";

const MODE_ID_STATIC: u8 = 0;
const MODE_ID_RAINBOW: u8 = 1;

pub trait Mode {
    /// Render a single frame
    fn render<const N: usize>(&mut self, now: Instant) -> [Rgb; N];

    /// Reset mode state
    fn reset(&mut self) {}

    /// Check if the mode is transitioning
    fn is_transitioning(&self) -> bool {
        false
    }
}

/// Mode slot - enum containing all possible modes
#[derive(Debug, Clone)]
pub enum ModeSlot {
    /// Rainbow cycling mode
    Rainbow(RainbowMode),
    /// Static single color
    Static(StaticColorMode),
}

/// Known mode ids that can be requested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ModeId {
    Static = MODE_ID_STATIC,
    Rainbow = MODE_ID_RAINBOW,
}

impl Default for ModeSlot {
    fn default() -> Self {
        Self::Rainbow(RainbowMode::default())
    }
}

impl ModeId {
    pub fn from_raw(value: u8) -> Option<Self> {
        Some(match value {
            MODE_ID_STATIC => Self::Static,
            MODE_ID_RAINBOW => Self::Rainbow,
            _ => return None,
        })
    }

    pub fn to_mode_slot(self, color: Rgb) -> ModeSlot {
        match self {
            Self::Static => ModeSlot::Static(StaticColorMode::new(color)),
            Self::Rainbow => ModeSlot::Rainbow(RainbowMode::default()),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Static => MODE_NAME_STATIC,
            Self::Rainbow => MODE_NAME_RAINBOW,
        }
    }

    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s {
            MODE_NAME_STATIC => Some(Self::Static),
            MODE_NAME_RAINBOW => Some(Self::Rainbow),
            _ => None,
        }
    }
}

impl ModeSlot {
    /// Render the current mode
    pub fn render<const N: usize>(&mut self, frame_time: Instant) -> [Rgb; N] {
        match self {
            Self::Rainbow(mode) => mode.render(frame_time),
            Self::Static(mode) => mode.render(frame_time),
        }
    }

    /// Reset the mode state
    pub fn reset(&mut self) {
        match self {
            Self::Rainbow(mode) => Mode::reset(mode),
            Self::Static(mode) => Mode::reset(mode),
        }
    }

    /// Get the mode ID for external observation
    pub fn mode_id(&self) -> ModeId {
        match self {
            Self::Rainbow(_) => ModeId::Rainbow,
            Self::Static(_) => ModeId::Static,
        }
    }

    /// Update the color of the current mode with optional transition.
    pub fn set_color(&mut self, color: Rgb, duration: Duration, now: Instant) {
        if let Self::Static(mode) = self {
            mode.set_color(color, duration, now);
        }
    }

    pub fn is_transitioning(&self) -> bool {
        if let Self::Static(mode) = self {
            mode.is_transitioning()
        } else {
            false
        }
    }
}
