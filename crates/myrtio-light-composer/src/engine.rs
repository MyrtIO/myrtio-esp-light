use embassy_time::{Duration, Instant, Timer};

#[cfg(feature = "log")]
use esp_println::println;

use crate::color::Rgb;
use crate::command::CommandReceiver;
use crate::effect::{EffectProcessor, EffectProcessorConfig};
use crate::mode::{ModeId, ModeSlot};
use crate::operation::OperationStack;
use crate::{Command, LedDriver, Operation};

const DEFAULT_FPS: u32 = 90;
const DEFAULT_FRAME_DURATION: Duration = Duration::from_millis(1000 / DEFAULT_FPS as u64);

/// Configuration for effect transitions
#[derive(Clone, Copy)]
pub struct TransitionConfig {
    /// Duration of fade-out phase
    pub fade_out_duration: Duration,
    /// Duration of fade-in phase
    pub fade_in_duration: Duration,
    /// Duration of color change
    pub color_change_duration: Duration,
    /// Duration of brightness change
    pub brightness_change_duration: Duration,
}

#[derive(Debug, Clone)]
struct LightState {
    color: Rgb,
    current_mode: ModeSlot,
    pending_mode: Option<ModeSlot>,
    brightness: u8,
}

/// Configuration for the light engine
#[derive(Clone)]
pub struct LightEngineConfig {
    pub mode: ModeId,
    pub effects: EffectProcessorConfig,
    pub transition_config: TransitionConfig,
    pub brightness: u8,
    pub color: Rgb,
}

/// Light Engine - the main orchestrator
pub struct LightEngine<D: LedDriver, const N: usize> {
    // External dependencies and configuration
    driver: D,
    commands: CommandReceiver,
    transition_config: TransitionConfig,

    // Internal state
    state: LightState,
    next_frame: Instant,
    stack: OperationStack<10>,

    // Internal dependencies
    effects: EffectProcessor,
}

impl<D: LedDriver, const N: usize> LightEngine<D, N> {
    /// Create a new light engine with command channel
    ///
    /// Returns the engine and a sender for commands.
    pub fn new(driver: D, commands: CommandReceiver, config: &LightEngineConfig) -> Self {
        let now = Instant::now();
        Self {
            driver,
            commands,
            transition_config: config.transition_config,
            state: LightState {
                color: config.color,
                current_mode: config.mode.to_mode_slot(config.color),
                pending_mode: None,
                brightness: config.brightness,
            },
            next_frame: now,
            stack: OperationStack::new(),
            effects: EffectProcessor::new(&config.effects),
        }
    }

    /// Process one frame
    ///
    /// This is the main render loop step. Call this continuously.
    pub async fn tick(&mut self) {
        self.next_frame += DEFAULT_FRAME_DURATION;
        let now = Instant::now();

        self.process_commands();
        self.process_operations(now);

        self.effects.tick(now);
        let mut frame: [Rgb; N] = self.state.current_mode.render(now);
        self.effects.apply(&mut frame);

        Timer::at(self.next_frame).await;
        self.driver.write(&frame);
    }

    /// Process pending commands from the channel (non-blocking)
    fn process_commands(&mut self) {
        while let Ok(cmd) = self.commands.try_receive() {
            let _result = match cmd {
                Command::SetBrightness(brightness) => self.stack.push_brightness(brightness),
                Command::SwitchMode(mode) => self.stack.push_mode(mode, self.state.brightness),
                Command::SetColor(color) => self.stack.push_color(color),
                Command::PowerOff => self.stack.push_power_off(),
                Command::PowerOn => self.stack.push_power_on(),
            };
            #[cfg(feature = "log")]
            if let Err(operation) = _result {
                println!(
                    "[light-engine.process_commands] error pushing operation: stack is full, dropping operation: {:?}",
                    operation
                );
            }
        }
    }

    /// Process the next operation from the stack
    fn process_operations(&mut self, now: Instant) {
        let Some(next) = self.process_current_operation() else {
            return;
        };
        // Start the transition for the current operation
        match next {
            Command::SetBrightness(brightness) => {
                self.effects.brightness.set(
                    brightness,
                    self.transition_config.brightness_change_duration,
                    now,
                );
            }
            Command::SetColor(color) => {
                self.state.current_mode.set_color(
                    color,
                    self.transition_config.color_change_duration,
                    now,
                );
            }
            Command::PowerOff => {
                self.effects.brightness.set(
                    0,
                    self.transition_config.brightness_change_duration,
                    now,
                );
            }
            Command::PowerOn => {
                self.effects.brightness.set(
                    self.state.brightness,
                    self.transition_config.brightness_change_duration,
                    now,
                );
            }
            Command::SwitchMode(_mode) => {
                // This command changes instantly
            }
        }
    }

    /// Process the current operation from the stack
    ///
    /// Returns the next operation to process
    fn process_current_operation(&mut self) -> Option<Operation> {
        let current = self.stack.current()?;
        let is_complete = match current {
            Operation::SetBrightness(_) | Operation::PowerOff | Operation::PowerOn => {
                !self.effects.brightness.is_transitioning()
            }
            Operation::SetColor(_) => !self.state.current_mode.is_transitioning(),
            Operation::SwitchMode(_) => true,
        };
        if !is_complete {
            return None;
        }
        // Apply the operation to the state
        match current {
            Operation::SetBrightness(brightness) => {
                self.state.brightness = brightness;
            }
            Operation::SetColor(color) => {
                self.state.color = color;
            }
            Operation::SwitchMode(mode) => {
                self.set_mode(mode);
            }
            Operation::PowerOff | Operation::PowerOn => {
                // This commands does not change the state
            }
        }

        self.stack.pop()
    }

    fn set_mode(&mut self, mode: ModeId) {
        let slot = mode.to_mode_slot(self.state.color);
        self.state.current_mode = slot;
        self.state.current_mode.reset();
        self.state.pending_mode = None;
    }
}
