use embassy_time::{Duration, Instant};

use crate::color::{Rgb, blend_colors};
use crate::math8::{blend8, progress8};

/// Blends two values of type `T` using a progress value (0-255)
pub type ValueBlender<T> = fn(T, T, u8) -> T;

/// Transition for values of type `T`
#[derive(Debug, Clone)]
pub struct ValueTransition<T: Copy> {
    /// Blender function
    blend: ValueBlender<T>,
    /// Current interpolated value
    current: T,
    /// Value at the start of transition
    source: T,
    /// Target value (None if no transition in progress)
    target: Option<T>,
    /// Total transition duration
    duration: Duration,
    /// Time at which the transition started
    start_time: Instant,
}

impl<T: Copy> ValueTransition<T> {
    /// Create a new value transition
    pub const fn new(initial: T, blend: ValueBlender<T>) -> Self {
        Self {
            blend,
            current: initial,
            source: initial,
            target: None,
            duration: Duration::from_millis(0),
            start_time: Instant::from_millis(0),
        }
    }

    /// Get current value
    pub const fn current(&self) -> T {
        self.current
    }

    /// Check if a transition is in progress
    pub const fn is_transitioning(&self) -> bool {
        self.target.is_some()
    }

    /// Set value for transition
    pub fn set(&mut self, value: T, duration: Duration, start_time: Instant) {
        self.start_time = start_time;
        if duration.as_millis() == 0 {
            // Immediate
            self.current = value;
            self.source = value;
            self.target = None;
            self.duration = Duration::from_millis(0);
        } else {
            // Smooth
            self.source = self.current;
            self.target = Some(value);
            self.duration = duration;
        }
    }

    /// Update transition state
    ///
    /// Call this once per frame with the frame delta time.
    pub fn tick(&mut self, now: Instant) {
        let Some(target) = self.target else {
            return;
        };

        let elapsed = now.duration_since(self.start_time);
        if elapsed >= self.duration {
            self.current = target;
            self.source = target;
            self.target = None;
            return;
        }

        let progress = progress8(elapsed, self.duration);
        self.current = (self.blend)(self.source, target, progress);
    }
}

impl ValueTransition<u8> {
    /// Create a new u8 transition
    pub const fn new_u8(initial: u8) -> Self {
        Self::new(initial, blend8)
    }
}

impl ValueTransition<Rgb> {
    /// Create a new rgb transition
    pub const fn new_rgb(initial: Rgb) -> Self {
        Self::new(initial, blend_colors)
    }
}
