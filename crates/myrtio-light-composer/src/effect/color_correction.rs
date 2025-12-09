//! Color correction effect
//!
//! Applies multiplicative color correction to each RGB channel.
//! Used for white balance and color temperature adjustments.

use crate::color::Rgb;
use crate::math8::scale8;

use super::Effect;

/// Color correction effect
///
/// Applies per-channel multiplicative scaling to correct color output.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ColorCorrection {
    /// Correction factors for each channel (0-255 = 0%-100%)
    factors: Rgb,
}

impl ColorCorrection {
    /// Create a new color correction from color
    pub(crate) const fn new(factors: Rgb) -> Self {
        Self { factors }
    }

    /// Check if correction is active
    pub(crate) const fn is_active(self) -> bool {
        self.factors.r != 255 || self.factors.g != 255 || self.factors.b != 255
    }
}

impl Effect for ColorCorrection {
    fn apply<const N: usize>(&mut self, frame: &mut [Rgb; N]) {
        if !self.is_active() {
            return;
        }

        // TODO: add gamma correction?
        for pixel in frame.iter_mut() {
            pixel.r = scale8(pixel.r, self.factors.r);
            pixel.g = scale8(pixel.g, self.factors.g);
            pixel.b = scale8(pixel.b, self.factors.b);
        }
    }
}
