use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, Receiver, Sender},
};
use embassy_time::{Duration, Instant, Timer};

use crate::ColorCorrection;

use super::{driver::LedDriver, effect::EffectSlot, processor::OutputProcessor};

/// Default frames per second
const DEFAULT_FPS: u32 = 90;

/// Command channel capacity
const COMMAND_CHANNEL_SIZE: usize = 4;

/// Commands that can be sent to the light engine
#[derive(Clone)]
pub enum Command<const N: usize> {
    /// Set brightness with transition duration.
    ///
    /// This updates both the target brightness and the current output brightness.
    SetBrightness { brightness: u8, duration: Duration },
    /// Set brightness immediately
    SetBrightnessImmediate(u8),
    /// Switch to a new effect with fade transition
    SwitchEffect(EffectSlot<N>),
    /// Switch effect instantly (no fade)
    SwitchEffectInstant(EffectSlot<N>),
    /// Update effect color
    SetColor {
        r: u8,
        g: u8,
        b: u8,
        duration: Duration,
    },
    /// Stop the engine (fade out)
    Stop(Duration),
    /// Start the engine (fade in)
    Start(Duration),
    /// Set transition configuration
    SetTransitionConfig(TransitionConfig),
    /// Power off the light (fade out to 0, but preserve target brightness).
    ///
    /// The target brightness stored in `SharedState` remains unchanged so that
    /// a subsequent `PowerOn` can restore it. Only the physical LED output
    /// fades to zero.
    PowerOff(Duration),
    /// Power on the light (fade in from 0 to the stored target brightness).
    ///
    /// Restores the brightness that was set before power-off.
    PowerOn(Duration),
}

/// Type alias for command sender
pub type CommandSender<const N: usize> =
    Sender<'static, CriticalSectionRawMutex, Command<N>, COMMAND_CHANNEL_SIZE>;

/// Type alias for command receiver  
pub type CommandReceiver<const N: usize> =
    Receiver<'static, CriticalSectionRawMutex, Command<N>, COMMAND_CHANNEL_SIZE>;

/// Type alias for the command channel
pub type CommandChannel<const N: usize> =
    Channel<CriticalSectionRawMutex, Command<N>, COMMAND_CHANNEL_SIZE>;

/// Engine state machine states
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    /// Normal operation - rendering current effect
    Running,
    /// Fading out before effect change
    FadingOut,
    /// Fading in after effect change
    FadingIn,
    /// Engine is stopped (LEDs off)
    Stopped,
}

/// Configuration for effect transitions
#[derive(Clone, Copy)]
pub struct TransitionConfig {
    /// Duration of fade-out phase
    pub fade_out_duration: Duration,
    /// Duration of fade-in phase
    pub fade_in_duration: Duration,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            fade_out_duration: Duration::from_millis(300),
            fade_in_duration: Duration::from_millis(300),
        }
    }
}

impl TransitionConfig {
    /// Create a symmetric transition (same fade-out and fade-in duration)
    pub fn symmetric(duration: Duration) -> Self {
        Self {
            fade_out_duration: duration,
            fade_in_duration: duration,
        }
    }

    /// Create an instant transition (no fade)
    pub fn instant() -> Self {
        Self {
            fade_out_duration: Duration::from_millis(0),
            fade_in_duration: Duration::from_millis(0),
        }
    }
}

/// Light Engine - the main orchestrator
///
/// Generic over `D: LedDriver` to support different hardware backends.
pub struct LightEngine<D: LedDriver<N>, const N: usize> {
    /// Hardware driver for LED output
    driver: D,
    /// Command receiver
    commands: CommandReceiver<N>,
    /// Current active effect
    current_effect: EffectSlot<N>,
    /// Pending effect (waiting for fade-out to complete)
    pending_effect: Option<EffectSlot<N>>,
    /// Output processor (brightness, etc.)
    processor: OutputProcessor<N>,
    /// Current engine state
    state: EngineState,
    /// Transition configuration
    transition_config: TransitionConfig,
    /// Target brightness (restored after fade-in)
    brightness: u8,
    /// Frame timing
    frame_duration: Duration,
    /// Start time for effect animations
    start_time: Instant,
    /// Next scheduled tick time for consistent FPS
    next_tick: Instant,
}

impl<D: LedDriver<N>, const N: usize> LightEngine<D, N> {
    /// Create a new light engine with command channel
    ///
    /// Returns the engine and a sender for commands.
    pub fn new(driver: D, commands: CommandReceiver<N>) -> Self {
        let now = Instant::now();
        let frame_duration = Duration::from_millis(1000 / u64::from(DEFAULT_FPS));
        Self {
            driver,
            commands,
            current_effect: EffectSlot::default(),
            pending_effect: None,
            processor: OutputProcessor::with_brightness(255, frame_duration),
            state: EngineState::Stopped,
            transition_config: TransitionConfig::default(),
            brightness: 255,
            frame_duration,
            start_time: now,
            next_tick: now,
        }
    }

    /// Set the output processor
    #[must_use]
    pub fn with_color_correction(mut self, color_correction: ColorCorrection) -> Self {
        self.processor.color_correction = color_correction;
        self
    }

    /// Set the output brightness scale
    #[must_use]
    pub fn with_brightness_scale(mut self, scale: u8) -> Self {
        self.processor.brightness.set_scale(scale);
        self
    }

    /// Set the target frame rate
    pub fn set_fps(&mut self, fps: u32) {
        self.frame_duration = Duration::from_millis(1000 / u64::from(fps));
    }

    /// Set transition configuration
    pub fn set_transition_config(&mut self, config: TransitionConfig) {
        self.transition_config = config;
    }

    /// Get current engine state
    pub fn state(&self) -> EngineState {
        self.state
    }

    /// Get current brightness
    pub fn brightness(&self) -> u8 {
        self.brightness
    }

    /// Set global brightness with transition
    pub fn set_brightness(&mut self, brightness: u8, duration: Duration) {
        self.brightness = brightness;
        self.processor.brightness.set(brightness, duration);
    }

    /// Set global brightness immediately
    pub fn set_brightness_immediate(&mut self, brightness: u8) {
        self.brightness = brightness;
        self.processor.brightness.set_immediate(brightness);
    }

    /// Switch to a new effect with fade transition
    pub fn switch_effect(&mut self, effect: EffectSlot<N>) {
        self.switch_effect_with_config(effect, self.transition_config);
    }

    /// Switch to a new effect with custom transition
    pub fn switch_effect_with_config(&mut self, effect: EffectSlot<N>, config: TransitionConfig) {
        match self.state {
            EngineState::Running => {
                if config.fade_out_duration.as_millis() == 0 {
                    // Instant switch
                    self.current_effect = effect;
                    self.current_effect.reset();
                } else {
                    // Start fade-out
                    self.pending_effect = Some(effect);
                    self.processor.brightness.fade_out(config.fade_out_duration);
                    self.state = EngineState::FadingOut;
                    self.transition_config = config;
                }
            }
            EngineState::FadingOut | EngineState::FadingIn => {
                // Replace pending effect
                self.pending_effect = Some(effect);
                self.transition_config = config;
            }
            EngineState::Stopped => {
                // Set effect and start fade-in
                self.current_effect = effect;
                self.current_effect.reset();
                self.processor
                    .brightness
                    .fade_in(self.brightness, config.fade_in_duration);
                self.state = EngineState::FadingIn;
            }
        }
    }

    /// Switch effect instantly (no fade)
    pub fn switch_effect_instant(&mut self, effect: EffectSlot<N>) {
        self.switch_effect_with_config(effect, TransitionConfig::instant());
    }

    pub fn set_effect(&mut self, effect: EffectSlot<N>) {
        self.current_effect = effect;
        self.current_effect.reset();
    }

    /// Stop the engine (fade out and turn off)
    pub fn stop(&mut self, fade_duration: Duration) {
        if fade_duration.as_millis() == 0 {
            self.processor.brightness.set_immediate(0);
            self.state = EngineState::Stopped;
        } else {
            self.processor.brightness.fade_out(fade_duration);
            self.state = EngineState::FadingOut;
            self.pending_effect = None; // Clear any pending effect
        }
    }

    /// Start the engine (fade in)
    pub fn start(&mut self, fade_duration: Duration) {
        if self.state == EngineState::Stopped {
            self.processor
                .brightness
                .fade_in(self.brightness, fade_duration);
            self.state = EngineState::FadingIn;
        }
    }

    /// Power off the light (fade out to 0, preserving target brightness).
    ///
    /// Unlike `stop`, this method does **not** modify `target_brightness`.
    /// The brightness stored in `SharedState` remains the same so that
    /// `power_on` can restore it later.
    pub fn power_off(&mut self, fade_duration: Duration) {
        // Delegate to stop for the actual fade-out logic.
        // target_brightness is intentionally left unchanged.
        self.stop(fade_duration);
    }

    /// Power on the light (fade in from 0 to the stored target brightness).
    ///
    /// Restores the brightness that was set before `power_off`.
    pub fn power_on(&mut self, fade_duration: Duration) {
        // Delegate to start which already fades in to target_brightness.
        self.start(fade_duration);
    }

    /// Process pending commands from the channel (non-blocking)
    fn process_commands(&mut self) {
        while let Ok(cmd) = self.commands.try_receive() {
            match cmd {
                Command::SetBrightness {
                    brightness,
                    duration,
                } => {
                    self.set_brightness(brightness, duration);
                }
                Command::SetBrightnessImmediate(brightness) => {
                    self.set_brightness_immediate(brightness);
                }
                Command::SwitchEffect(effect) => {
                    self.switch_effect(effect);
                }
                Command::SwitchEffectInstant(effect) => {
                    self.switch_effect_instant(effect);
                }
                Command::SetColor { r, g, b, duration } => {
                    self.current_effect.set_color_rgb(r, g, b, duration);
                }
                Command::Stop(duration) => {
                    self.stop(duration);
                }
                Command::Start(duration) => {
                    self.start(duration);
                }
                Command::SetTransitionConfig(config) => {
                    self.set_transition_config(config);
                }
                Command::PowerOff(duration) => {
                    self.power_off(duration);
                }
                Command::PowerOn(duration) => {
                    self.power_on(duration);
                }
            }
        }
    }

    /// Process one frame
    ///
    /// This is the main render loop step. Call this continuously.
    pub async fn tick(&mut self) {
        // Schedule target time for this frame completion
        self.next_tick += self.frame_duration;

        // Process any pending commands
        self.process_commands();

        // Update processor state (transitions)
        self.processor.tick();

        // Handle state machine transitions
        self.update_state();

        // Get elapsed time since engine start (for effect animations)
        let elapsed = self.start_time.elapsed();

        // Render current effect
        let mut frame = self.current_effect.render(elapsed);

        // Apply post-processing
        self.processor.apply(&mut frame);

        // Write to hardware
        self.driver.write(&frame);

        // Wait until the scheduled time to maintain constant FPS
        Timer::at(self.next_tick).await;

        // Drift correction: if we missed the window significantly,
        // reset the schedule to avoid trying to catch up too aggressively.
        let now = Instant::now();
        // Allow up to 3 frames of lag before resetting
        if now > self.next_tick + (self.frame_duration * 3) {
            self.next_tick = now;
        }
    }

    /// Run the engine loop indefinitely
    pub async fn run(&mut self) -> ! {
        loop {
            self.tick().await;
        }
    }

    /// Update state machine based on transition progress
    fn update_state(&mut self) {
        match self.state {
            EngineState::FadingOut => {
                if self.processor.brightness.is_faded_out() {
                    // Fade-out complete
                    if let Some(effect) = self.pending_effect.take() {
                        // Switch to new effect and start fade-in
                        self.current_effect = effect;
                        self.current_effect.reset();
                        self.processor
                            .brightness
                            .fade_in(self.brightness, self.transition_config.fade_in_duration);
                        self.state = EngineState::FadingIn;
                    } else {
                        // No pending effect - engine stopped
                        self.state = EngineState::Stopped;
                    }
                }
            }
            EngineState::FadingIn => {
                if !self.processor.brightness.is_transitioning() {
                    // Fade-in complete
                    self.state = EngineState::Running;
                }
            }
            EngineState::Running | EngineState::Stopped => {
                // No state transition needed
            }
        }
    }
}
