//! Output processing pipeline
//!
//! Applies post-processing to rendered frames before sending to hardware.
//! Currently includes:
//! - Brightness envelope (for fades and global brightness)
//!
//! Future additions could include:
//! - Gamma correction
//! - Color temperature adjustment
//! - Dithering

mod brightness;

pub use brightness::BrightnessEnvelope;

use smart_leds::RGB;

/// Output processor - applies post-processing to frames
///
/// This is the central hub for all output modifications.
/// Processing is applied in a specific order to ensure correct results.
#[derive(Default)]
pub struct OutputProcessor<const N: usize> {
    /// Brightness envelope for fades
    pub brightness: BrightnessEnvelope<N>,
}

impl<const N: usize> OutputProcessor<N> {
    /// Create a new output processor with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific initial brightness
    pub fn with_brightness(brightness: u8) -> Self {
        Self {
            brightness: BrightnessEnvelope::new(brightness),
        }
    }

    /// Apply all processing to a frame
    ///
    /// Processing order:
    /// 1. Brightness scaling
    /// 2. (Future: Gamma correction)
    /// 3. (Future: Color temperature)
    pub fn apply(&self, frame: &mut [RGB<u8>; N]) {
        self.brightness.apply(frame);
    }

    /// Update processor state (call each frame)
    pub fn tick(&mut self) {
        self.brightness.tick();
    }

    /// Check if any transitions are in progress
    pub fn is_transitioning(&self) -> bool {
        self.brightness.is_transitioning()
    }
}
