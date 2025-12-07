//! Shared light state for external observation
//!
//! Provides a way to observe the engine's current state from outside
//! without direct access to the engine instance.
//!
//! ## Brightness Semantics
//!
//! The `brightness` field represents the **target brightness** — the brightness
//! level the light will have when powered on. This value remains stable across
//! power-off/power-on cycles:
//!
//! - When the light is powered off, `brightness` still holds the last set target.
//! - When the light is powered on, it fades in to this target brightness.
//! - The `is_on` field indicates whether the light is currently powered on.
//!
//! This design allows external systems (e.g., Home Assistant) to display and
//! control the brightness slider even when the light is off.

use core::sync::atomic::{AtomicU8, Ordering};

use crate::LightSnapshot;

/// Effect identifier for external observation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EffectId {
    Off = 0,
    Rainbow = 1,
    Static = 2,
    RainbowFlow = 3,
}

impl From<u8> for EffectId {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Rainbow,
            2 => Self::Static,
            3 => Self::RainbowFlow,
            _ => Self::Off,
        }
    }
}

/// Shared light state that can be observed from outside the engine
///
/// Uses atomics for lock-free thread-safe access.
/// The engine updates this state, and external code can read it.
///
/// Note: `brightness` is the **target brightness**, not the instantaneous
/// output level. It remains constant when the light is powered off, so
/// external systems can display/control brightness even while off.
pub struct SharedState {
    /// Target brightness (0-255).
    ///
    /// This is the brightness the light will have when on, not the current
    /// physical output (which may be 0 when powered off).
    brightness: AtomicU8,
    /// Whether the light is on (1) or off (0).
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

    /// Set the light state from a [`LightSnapshot`]
    pub fn set_from_snapshot(&self, snapshot: LightSnapshot) {
        self.set_brightness(snapshot.brightness);
        self.set_on(snapshot.is_on);
        self.set_effect(snapshot.effect);
        self.set_rgb(snapshot.r, snapshot.g, snapshot.b);
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}
