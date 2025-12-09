use core::sync::atomic::{AtomicU8, Ordering};

use myrtio_light_composer::{Command, CommandSender, ModeId, Rgb};

use crate::domain::dto::LightChangeIntent;
use crate::domain::entity::LightState;
use crate::domain::ports::{LightIntentApplier, LightStateHandler, LightStateReader};

/// Atomic light state
/// Uses atomics for lock-free thread-safe access.
#[derive(Debug)]
struct AtomicLightState {
    power: AtomicU8,
    brightness: AtomicU8,
    effect_id: AtomicU8,
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
            r: AtomicU8::new(state.color.0),
            g: AtomicU8::new(state.color.1),
            b: AtomicU8::new(state.color.2),
        }
    }

    fn get(&self) -> LightState {
        let r = self.r.load(Ordering::Relaxed);
        let g = self.g.load(Ordering::Relaxed);
        let b = self.b.load(Ordering::Relaxed);

        LightState {
            brightness: self.brightness.load(Ordering::Relaxed),
            power: self.power.load(Ordering::Relaxed) != 0,
            mode_id: self.effect_id.load(Ordering::Relaxed),
            color: (r, g, b),
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

pub(crate) struct LightStateService {
    cmd_sender: CommandSender,
}

impl LightStateService {
    pub(crate) fn new(cmd_sender: CommandSender) -> Self {
        Self { cmd_sender }
    }

    fn send_command(&self, command: Command) -> Result<(), ()> {
        self.cmd_sender.try_send(command).map_err(|_| ())
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
            let mode_id = ModeId::from_raw(mode_id_raw).expect("Invalid mode ID");
            state.mode_id = mode_id_raw;
            self.send_command(Command::SwitchMode(mode_id))?;
        }

        if let Some(brightness) = intent.brightness {
            state.brightness = brightness;
            self.send_command(Command::SetBrightness(brightness))?;
        }

        if let Some((r, g, b)) = intent.color {
            state.color = (r, g, b);
            self.send_command(Command::SetColor(Rgb { r, g, b }))?;
        }

        if intent.is_off() {
            state.power = false;
            self.send_command(Command::PowerOff)?;
        } else if intent.implies_on() && !state.power {
            state.power = true;
            self.send_command(Command::PowerOn)?;
        }

        LIGHT_STATE.set(&state);

        Ok(())
    }
}

impl LightStateHandler for LightStateService {}
