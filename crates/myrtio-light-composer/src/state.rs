//! Shared light state for external observation
//!
//! Provides a way to observe the engine's current state from outside
//! without direct access to the engine instance.

use core::sync::atomic::{AtomicU8, Ordering};

/// Effect identifier for external observation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EffectId {
    Off = 0,
    Rainbow = 1,
    Static = 2,
}

impl From<u8> for EffectId {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Rainbow,
            2 => Self::Static,
            _ => Self::Off,
        }
    }
}

/// Shared light state that can be observed from outside the engine
///
/// Uses atomics for lock-free thread-safe access.
/// The engine updates this state, and external code can read it.
pub struct SharedState {
    /// Target brightness (0-255)
    brightness: AtomicU8,
    /// Whether the light is on
    is_on: AtomicU8,
    /// Current effect ID
    effect: AtomicU8,
    /// RGB color components
    r: AtomicU8,
    g: AtomicU8,
    b: AtomicU8,
}

impl SharedState {
    /// Create a new shared state with default values
    pub const fn new() -> Self {
        Self {
            brightness: AtomicU8::new(255),
            is_on: AtomicU8::new(0),
            effect: AtomicU8::new(0),
            r: AtomicU8::new(255),
            g: AtomicU8::new(255),
            b: AtomicU8::new(255),
        }
    }

    // === Read methods (for external observation) ===

    /// Get current brightness
    pub fn brightness(&self) -> u8 {
        self.brightness.load(Ordering::Relaxed)
    }

    /// Check if light is on
    pub fn is_on(&self) -> bool {
        self.is_on.load(Ordering::Relaxed) != 0
    }

    /// Get current effect ID
    pub fn effect(&self) -> EffectId {
        self.effect.load(Ordering::Relaxed).into()
    }

    /// Get current RGB color
    pub fn rgb(&self) -> (u8, u8, u8) {
        (
            self.r.load(Ordering::Relaxed),
            self.g.load(Ordering::Relaxed),
            self.b.load(Ordering::Relaxed),
        )
    }

    // === Write methods (for engine to update) ===

    /// Set brightness
    pub fn set_brightness(&self, value: u8) {
        self.brightness.store(value, Ordering::Relaxed);
    }

    /// Set on/off state
    pub fn set_on(&self, on: bool) {
        self.is_on.store(u8::from(on), Ordering::Relaxed);
    }

    /// Set effect ID
    pub fn set_effect(&self, effect: EffectId) {
        self.effect.store(effect as u8, Ordering::Relaxed);
    }

    /// Set RGB color
    pub fn set_rgb(&self, r: u8, g: u8, b: u8) {
        self.r.store(r, Ordering::Relaxed);
        self.g.store(g, Ordering::Relaxed);
        self.b.store(b, Ordering::Relaxed);
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}
