use embassy_time::Instant;

use crate::color::Rgb;

mod brightness;
mod color_correction;

pub(crate) trait Effect {
    /// Apply the effect to a frame
    fn apply<const N: usize>(&mut self, frame: &mut [Rgb; N]);

    fn tick(&mut self, _now: Instant) {}
}

use brightness::BrightnessEffect;
pub use brightness::BrightnessEffectConfig;
use color_correction::ColorCorrection;

#[derive(Debug, Clone)]
pub struct EffectProcessorConfig {
    /// Brightness effect
    pub brightness: BrightnessEffectConfig,
    /// Color correction
    pub color_correction: Option<Rgb>,
}

/// Effect processor - applies post-processing to frames
///
/// This is the central hub for all output modifications.
/// Processing is applied in a specific order to ensure correct results.
#[derive(Debug)]
pub(crate) struct EffectProcessor {
    /// Brightness effect
    pub brightness: BrightnessEffect,
    /// Color correction effect
    pub color_correction: Option<ColorCorrection>,
}

impl EffectProcessor {
    /// Create a new output processor with default settings
    pub(crate) fn new(config: &EffectProcessorConfig) -> Self {
        let brightness = BrightnessEffect::new(0, &config.brightness);
        let color_correction = config.color_correction.map(ColorCorrection::new);
        Self {
            brightness,
            color_correction,
        }
    }

    /// Apply all processing to a frame
    pub(crate) fn apply<const N: usize>(&mut self, frame: &mut [Rgb; N]) {
        if let Some(color_correction) = &mut self.color_correction {
            color_correction.apply(frame);
        }
        self.brightness.apply(frame);
    }

    /// Tick the effects
    pub(crate) fn tick(&mut self, now: Instant) {
        self.brightness.tick(now);
        if let Some(color_correction) = &mut self.color_correction {
            color_correction.tick(now);
        }
    }
}
