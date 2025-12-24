use core::sync::atomic::{AtomicU8, AtomicU16, Ordering};

use crate::domain::entity::{ColorMode, LightState};

/// Atomic light state
/// Uses atomics for lock-free thread-safe access.
#[derive(Debug)]
pub(super) struct AtomicLightState {
    power: AtomicU8,
    brightness: AtomicU8,
    effect_id: AtomicU8,
    color_temp: AtomicU16,
    color_mode: AtomicU8,
    r: AtomicU8,
    g: AtomicU8,
    b: AtomicU8,
}

impl AtomicLightState {
    pub(super) const fn from_state(state: &LightState) -> Self {
        Self {
            power: AtomicU8::new(if state.power { 1 } else { 0 }),
            brightness: AtomicU8::new(state.brightness),
            effect_id: AtomicU8::new(state.mode_id),
            color_temp: AtomicU16::new(state.color_temp),
            color_mode: AtomicU8::new(state.color_mode.as_u8()),
            r: AtomicU8::new(state.color.0),
            g: AtomicU8::new(state.color.1),
            b: AtomicU8::new(state.color.2),
        }
    }

    pub(super) fn get(&self) -> LightState {
        let r = self.r.load(Ordering::Relaxed);
        let g = self.g.load(Ordering::Relaxed);
        let b = self.b.load(Ordering::Relaxed);

        let color_mode_raw = self.color_mode.load(Ordering::Relaxed);
        let color_mode = ColorMode::from_u8(color_mode_raw).unwrap();

        LightState {
            power: self.power.load(Ordering::Relaxed) != 0,
            brightness: self.brightness.load(Ordering::Relaxed),
            color: (r, g, b),
            color_temp: self.color_temp.load(Ordering::Relaxed),
            mode_id: self.effect_id.load(Ordering::Relaxed),
            color_mode,
        }
    }

    pub(super) fn set(&self, state: &LightState) {
        self.brightness.store(state.brightness, Ordering::Relaxed);
        self.power.store(u8::from(state.power), Ordering::Relaxed);
        self.effect_id.store(state.mode_id, Ordering::Relaxed);
        self.r.store(state.color.0, Ordering::Relaxed);
        self.g.store(state.color.1, Ordering::Relaxed);
        self.b.store(state.color.2, Ordering::Relaxed);
    }
}
