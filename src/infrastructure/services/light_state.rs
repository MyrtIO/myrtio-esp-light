use core::sync::atomic::{AtomicU8, AtomicU16, Ordering};

use myrtio_light_composer::{IntentSender, LightIntent, ModeId, Rgb};

use crate::domain::dto::LightChangeIntent;
use crate::domain::entity::{ColorMode, LightState};
use crate::domain::ports::{LightIntentApplier, LightStateHandler, LightStateReader};

/// Atomic light state
/// Uses atomics for lock-free thread-safe access.
#[derive(Debug)]
struct AtomicLightState {
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
    const fn from_state(state: &LightState) -> Self {
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

    fn get(&self) -> LightState {
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

    fn set(&self, state: &LightState) {
        self.brightness.store(state.brightness, Ordering::Relaxed);
        self.power.store(u8::from(state.power), Ordering::Relaxed);
        self.effect_id.store(state.mode_id, Ordering::Relaxed);
        self.r.store(state.color.0, Ordering::Relaxed);
        self.g.store(state.color.1, Ordering::Relaxed);
        self.b.store(state.color.2, Ordering::Relaxed);
    }
}

/// Global thread-safe lock-free light state
static LIGHT_STATE: AtomicLightState = AtomicLightState::from_state(&LightState::new());

pub struct LightStateService {
    intents: IntentSender,
}

impl LightStateService {
    pub fn new(intents: IntentSender) -> Self {
        Self { intents }
    }

    fn send(&self, intent: LightIntent) -> Result<(), ()> {
        self.intents.try_send(intent).map_err(|_| ())
    }
}

impl LightStateReader for LightStateService {
    fn get_light_state(&self) -> Option<LightState> {
        Some(LIGHT_STATE.get())
    }
}

impl LightIntentApplier for LightStateService {
    fn apply_intent(&mut self, intent: LightChangeIntent) -> Result<(), ()> {
        let mut state = LIGHT_STATE.get();
        if let Some(mode_id_raw) = intent.mode_id {
            state.mode_id = mode_id_raw;
        }
        if let Some(brightness) = intent.brightness {
            state.brightness = brightness;
        }
        if let Some((r, g, b)) = intent.color {
            state.color = (r, g, b);
            state.color_mode = ColorMode::Rgb;
        } else if let Some(color_temp) = intent.color_temp {
            state.color_temp = color_temp;
            state.color_mode = ColorMode::Temperature;
        }

        if let Some(power) = intent.power {
            state.power = power;
        }

        let composer_intent = LightIntent {
            power: intent.power,
            brightness: intent.brightness,
            color: intent.color.map(|(r, g, b)| Rgb { r, g, b }),
            color_temperature: intent.color_temp,
            mode_id: intent.mode_id.and_then(ModeId::from_raw),
        };

        LIGHT_STATE.set(&state);

        self.send(composer_intent)
    }
}

impl LightStateHandler for LightStateService {}
