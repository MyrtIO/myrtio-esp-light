//! Output processing pipeline
//!
//! Applies post-processing to rendered frames before sending to hardware.
//! Currently includes:
//! - Brightness envelope (for fades and global brightness)
//! - Color correction (for white balance adjustments)
//!
//! Future additions could include:
//! - Gamma correction
//! - Dithering

mod brightness;
mod color_correction;

pub use brightness::BrightnessEnvelope;
pub use color_correction::ColorCorrection;

use embassy_time::Duration;
use smart_leds::RGB;

/// Output processor - applies post-processing to frames
///
/// This is the central hub for all output modifications.
/// Processing is applied in a specific order to ensure correct results.
pub struct OutputProcessor<const N: usize> {
    /// Brightness envelope for fades
    pub brightness: BrightnessEnvelope<N>,
    /// Color correction for white balance
    pub color_correction: ColorCorrection,
}

impl<const N: usize> Default for OutputProcessor<N> {
    fn default() -> Self {
        Self {
            brightness: BrightnessEnvelope::default(),
            color_correction: ColorCorrection::default(),
        }
    }
}

impl<const N: usize> OutputProcessor<N> {
    /// Create a new output processor with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific initial brightness
    pub fn with_brightness(brightness: u8, frame_time: Duration) -> Self {
        Self {
            brightness: BrightnessEnvelope::new(brightness, frame_time),
            color_correction: ColorCorrection::default(),
        }
    }

    /// Apply all processing to a frame
    ///
    /// Processing order:
    /// 1. Brightness scaling
    /// 2. Color correction
    pub fn apply(&self, frame: &mut [RGB<u8>; N]) {
        self.brightness.apply(frame);
        self.color_correction.apply(frame);
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
