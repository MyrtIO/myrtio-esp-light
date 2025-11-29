//! Color correction processor
//!
//! Applies multiplicative color correction to each RGB channel.
//! Used for white balance and color temperature adjustments.
//!
//! The correction color represents the scaling factors for each channel:
//! - 0xFFFFFF = no correction (100% for all channels)
//! - 0xFFAA78 = R at 100%, G at 67%, B at 47%

use smart_leds::RGB;

/// Color correction processor
///
/// Applies per-channel multiplicative scaling to correct color output.
/// Default is no correction (all channels at 100%).
#[derive(Clone, Copy)]
pub struct ColorCorrection {
    /// Correction factors for each channel (0-255 = 0%-100%)
    factors: RGB<u8>,
}

impl Default for ColorCorrection {
    fn default() -> Self {
        Self {
            factors: RGB { r: 255, g: 255, b: 255 },
        }
    }
}

impl ColorCorrection {
    /// Create a new color correction with no correction applied
    pub fn new() -> Self {
        Self::default()
    }

    /// Create color correction from a u32 color value (0xRRGGBB format)
    pub fn from_rgb(color: u32) -> Self {
        let r = ((color >> 16) & 0xFF) as u8;
        let g = ((color >> 8) & 0xFF) as u8;
        let b = (color & 0xFF) as u8;
        Self {
            factors: RGB { r, g, b },
        }
    }

    /// Set correction from a u32 color value (0xRRGGBB format)
    ///
    /// Example: `0xFFAA78` will scale R by 100%, G by 67%, B by 47%
    pub fn set(&mut self, color: u32) {
        self.factors.r = ((color >> 16) & 0xFF) as u8;
        self.factors.g = ((color >> 8) & 0xFF) as u8;
        self.factors.b = (color & 0xFF) as u8;
    }

    /// Set correction from individual RGB values
    pub fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.factors.r = r;
        self.factors.g = g;
        self.factors.b = b;
    }

    /// Get current correction factors
    pub fn factors(&self) -> RGB<u8> {
        self.factors
    }

    /// Check if correction is active (not all channels at 100%)
    pub fn is_active(&self) -> bool {
        self.factors.r != 255 || self.factors.g != 255 || self.factors.b != 255
    }

    /// Apply color correction to a frame
    pub fn apply<const N: usize>(&self, frame: &mut [RGB<u8>; N]) {
        // Skip if no correction needed
        if !self.is_active() {
            return;
        }

        for pixel in frame.iter_mut() {
            pixel.r = scale8(pixel.r, self.factors.r);
            pixel.g = scale8(pixel.g, self.factors.g);
            pixel.b = scale8(pixel.b, self.factors.b);
        }
    }
}

/// Scale an 8-bit value by a factor (0-255 = 0.0-1.0)
///
/// Uses integer math for efficiency on embedded systems.
#[inline]
fn scale8(value: u8, scale: u8) -> u8 {
    ((u16::from(value) * u16::from(scale)) >> 8) as u8
}

