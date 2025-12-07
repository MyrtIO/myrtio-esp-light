use core::sync::atomic::{AtomicU8, Ordering};

use embassy_time::Duration;
use myrtio_light_composer::effect::EffectName;
use myrtio_light_composer::{Command, EffectId};

use crate::domain::dto::LightChangeIntent;
use crate::domain::entity::LightState;
use crate::domain::ports::{LightIntentApplier, LightStateHandler, LightStateReader};
use crate::infrastructure::types::{LightCommand, LightCommandSender};

/// Atomic light state
/// Uses atomics for lock-free thread-safe access.
#[derive(Debug, Default)]
struct AtomicLightState {
    power: AtomicU8,
    brightness: AtomicU8,
    effect_id: AtomicU8,
    r: AtomicU8,
    g: AtomicU8,
    b: AtomicU8,
}

impl AtomicLightState {
    const fn new() -> Self {
        Self {
            power: AtomicU8::new(0),
            brightness: AtomicU8::new(0),
            effect_id: AtomicU8::new(0),
            r: AtomicU8::new(0),
            g: AtomicU8::new(0),
            b: AtomicU8::new(0),
        }
    }

    fn get(&self) -> LightState {
        let r = self.r.load(Ordering::Relaxed);
        let g = self.g.load(Ordering::Relaxed);
        let b = self.b.load(Ordering::Relaxed);

        LightState {
            brightness: self.brightness.load(Ordering::Relaxed),
            power: self.power.load(Ordering::Relaxed) != 0,
            effect_id: self.effect_id.load(Ordering::Relaxed),
            color: (r, g, b),
        }
    }

    fn set(&self, state: &LightState) {
        self.brightness.store(state.brightness, Ordering::Relaxed);
        self.power.store(u8::from(state.power), Ordering::Relaxed);
        self.effect_id.store(state.effect_id, Ordering::Relaxed);
        self.r.store(state.color.0, Ordering::Relaxed);
        self.g.store(state.color.1, Ordering::Relaxed);
        self.b.store(state.color.2, Ordering::Relaxed);
    }
}

/// Global thread-safe lock-free light state
static LIGHT_STATE: AtomicLightState = AtomicLightState::new();

pub(crate) struct LightStateService {
    cmd_sender: LightCommandSender,
}

impl LightStateService {
    pub(crate) fn new(cmd_sender: LightCommandSender) -> Self {
        Self { cmd_sender }
    }

    fn send_command(&self, command: LightCommand) -> Result<(), ()> {
        self.cmd_sender.try_send(command).map_err(|_| ())
    }
}

impl LightStateReader for LightStateService {
    fn get_light_state(&self) -> Option<LightState> {
        Some(LIGHT_STATE.get())
    }
}

const BRIGHTNESS_TRANSITION_MS: u64 = 400;
const COLOR_TRANSITION_MS: u64 = 300;

impl LightIntentApplier for LightStateService {
    fn apply_intent(&mut self, intent: LightChangeIntent) -> Result<(), ()> {
        let mut state = LIGHT_STATE.get();
        let brightness_duration = Duration::from_millis(BRIGHTNESS_TRANSITION_MS);

        if let Some(effect_id_raw) = intent.effect {
            let effect_id = EffectId::from(effect_id_raw);
            let effect_name = EffectName::from_id(effect_id).expect("Invalid effect ID");
            let effect_slot =
                effect_name.to_effect_slot(state.color.0, state.color.1, state.color.2);
            state.effect_id = effect_id_raw;

            self.send_command(Command::SwitchEffect(effect_slot))?;
        }
        if let Some((r, g, b)) = intent.color {
            state.color = (r, g, b);
            self.send_command(Command::SetColor {
                r,
                g,
                b,
                duration: Duration::from_millis(COLOR_TRANSITION_MS),
            })?;
        }

        if let Some(brightness) = intent.brightness {
            state.brightness = brightness;
            self.send_command(Command::SetBrightness {
                brightness,
                duration: brightness_duration,
            })?;
        }

        if intent.is_off() {
            state.power = false;
            self.send_command(Command::PowerOff(brightness_duration))?;
        } else if intent.implies_on() {
            state.power = true;
            self.send_command(Command::PowerOn(brightness_duration))?;
        }

        LIGHT_STATE.set(&state);

        Ok(())
    }
}

impl LightStateHandler for LightStateService {}
