use crate::{EffectId, SharedState};

/// A snapshot of the current light state for external systems.
///
/// This DTO captures the observable state without exposing domain internals.
#[derive(Clone, Copy, Debug)]
pub struct LightSnapshot {
    /// Whether the light is currently on
    pub is_on: bool,
    /// Current brightness level (0-255)
    pub brightness: u8,
    /// Current effect identifier
    pub effect: EffectId,
    /// Red component of current color
    pub r: u8,
    /// Green component of current color
    pub g: u8,
    /// Blue component of current color
    pub b: u8,
}

impl Default for LightSnapshot {
    fn default() -> Self {
        Self {
            is_on: true,
            brightness: 255,
            effect: EffectId::Rainbow,
            r: 255,
            g: 255,
            b: 255,
        }
    }
}

impl LightSnapshot {
    /// Create a snapshot from the shared state
    pub fn from_shared(shared: &SharedState) -> Self {
        let (r, g, b) = shared.rgb();
        Self {
            is_on: shared.is_on(),
            brightness: shared.brightness(),
            effect: shared.effect(),
            r,
            g,
            b,
        }
    }
}
