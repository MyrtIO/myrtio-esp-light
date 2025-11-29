//! Rainbow cycling effects
//!
//! Provides two rainbow effect variants:
//! - `RainbowEffect`: Uses fixed-point HSV gradient algorithm (ported from FastLED)
//! - `RainbowFlowEffect`: Three-point mirrored gradient with smooth flow

use core::cmp::min;
use embassy_time::Duration;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    RGB,
};

use super::EffectImpl;

const DEFAULT_CYCLE_MS: u64 = 12_000;
const HUE_STEP: u8 = 60;

// =============================================================================
// RainbowEffect - Fixed-point HSV gradient (ported from C++ FastLED)
// =============================================================================

/// Rainbow effect using fixed-point HSV gradient algorithm
///
/// This implementation is ported from the FastLED `fillGradient` function
/// and uses 8.24 fixed-point arithmetic for smooth color transitions.
#[derive(Clone)]
pub struct RainbowEffect {
    /// Duration of one complete rainbow cycle
    cycle_duration: Duration,
    /// Brightness value (0-255)
    value: u8,
    /// Saturation (0-255)
    saturation: u8,
}

impl Default for RainbowEffect {
    fn default() -> Self {
        Self {
            cycle_duration: Duration::from_millis(DEFAULT_CYCLE_MS),
            value: 255,
            saturation: 255,
        }
    }
}

impl RainbowEffect {
    /// Create a new rainbow effect with custom parameters
    pub fn new(cycle_duration: Duration, value: u8, saturation: u8) -> Self {
        Self {
            cycle_duration,
            value,
            saturation,
        }
    }

    /// Set the cycle duration
    #[must_use]
    pub fn with_cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration;
        self
    }

    /// Set the brightness value
    #[must_use]
    pub fn with_value(mut self, value: u8) -> Self {
        self.value = value;
        self
    }

    /// Set the saturation
    #[must_use]
    pub fn with_saturation(mut self, saturation: u8) -> Self {
        self.saturation = saturation;
        self
    }
}

impl<const N: usize> EffectImpl<N> for RainbowEffect {
    fn render(&mut self, time: Duration) -> [RGB<u8>; N] {
        let mut leds = [RGB::default(); N];
        if N == 0 {
            return leds;
        }

        let cycle_ms = self.cycle_duration.as_millis().max(1);
        let progress_ms = time.as_millis() % cycle_ms;
        #[allow(clippy::cast_possible_truncation)]
        let base_hue = ((progress_ms * 255) / cycle_ms) as u8;

        let c1 = Hsv {
            hue: base_hue,
            sat: self.saturation,
            val: self.value,
        };
        let c2 = Hsv {
            hue: base_hue.wrapping_add(HUE_STEP),
            sat: self.saturation,
            val: self.value,
        };
        let c3 = Hsv {
            hue: base_hue.wrapping_add(HUE_STEP * 2),
            sat: self.saturation,
            val: self.value,
        };

        // Compute center for mirroring
        let mut center_len = N / 2;
        if N % 2 != 0 {
            center_len += 1;
        }
        center_len = min(center_len, N);

        // Fill first half with three-point gradient using fixed-point math
        {
            let (first_half, _) = leds.split_at_mut(center_len);
            fill_gradient_three_fp(first_half, c1, c2, c3);
        }

        // Mirror to second half
        mirror_half(&mut leds, center_len);
        leds
    }

    fn reset(&mut self) {
        // Rainbow effect is stateless relative to start time
    }
}

// =============================================================================
// RainbowFlowEffect - Three-point mirrored gradient with floating-point math
// =============================================================================

/// Rainbow flow effect with three-point mirrored gradient
///
/// Uses floating-point interpolation for smooth color transitions.
#[derive(Clone)]
pub struct RainbowFlowEffect {
    /// Duration of one complete rainbow cycle
    cycle_duration: Duration,
    /// Brightness value (0-255)
    value: u8,
    /// Saturation (0-255)
    saturation: u8,
}

impl Default for RainbowFlowEffect {
    fn default() -> Self {
        Self {
            cycle_duration: Duration::from_millis(DEFAULT_CYCLE_MS),
            value: 255,
            saturation: 255,
        }
    }
}

impl RainbowFlowEffect {
    /// Create a new rainbow flow effect with custom parameters
    pub fn new(cycle_duration: Duration, value: u8, saturation: u8) -> Self {
        Self {
            cycle_duration,
            value,
            saturation,
        }
    }

    /// Set the cycle duration
    #[must_use]
    pub fn with_cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration;
        self
    }

    /// Set the brightness value
    #[must_use]
    pub fn with_value(mut self, value: u8) -> Self {
        self.value = value;
        self
    }

    /// Set the saturation
    #[must_use]
    pub fn with_saturation(mut self, saturation: u8) -> Self {
        self.saturation = saturation;
        self
    }
}

impl<const N: usize> EffectImpl<N> for RainbowFlowEffect {
    fn render(&mut self, time: Duration) -> [RGB<u8>; N] {
        let mut leds = [RGB::default(); N];
        if N == 0 {
            return leds;
        }

        let cycle_ms = self.cycle_duration.as_millis().max(1);
        let progress_ms = time.as_millis() % cycle_ms;
        #[allow(clippy::cast_possible_truncation)]
        let base_hue = ((progress_ms * 255) / cycle_ms) as u8;

        let color1 = Hsv {
            hue: base_hue,
            sat: self.saturation,
            val: self.value,
        };
        let color2 = Hsv {
            hue: base_hue.wrapping_add(HUE_STEP),
            sat: self.saturation,
            val: self.value,
        };
        let color3 = Hsv {
            hue: base_hue.wrapping_add(HUE_STEP * 2),
            sat: self.saturation,
            val: self.value,
        };

        let mut center_len = N / 2;
        if N % 2 != 0 {
            center_len += 1;
        }
        center_len = min(center_len, N);

        {
            let (first_half, _) = leds.split_at_mut(center_len);
            fill_gradient_three_float(first_half, color1, color2, color3, HueDirection::Forward);
        }

        mirror_half(&mut leds, center_len);
        leds
    }

    fn reset(&mut self) {
        // Rainbow flow effect is stateless relative to start time
    }
}

// =============================================================================
// Fixed-point gradient algorithm (ported from FastLED/C++)
// =============================================================================

/// Hue direction for gradient calculation
#[derive(Clone, Copy, PartialEq, Eq)]
enum GradientDirection {
    Forward,
    Backward,
    Shortest,
}

/// Fill gradient using fixed-point 8.24 arithmetic (ported from FastLED)
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_lossless)]
fn fill_gradient_fp(
    leds: &mut [RGB<u8>],
    start_pos: usize,
    start_color: Hsv,
    end_pos: usize,
    end_color: Hsv,
    direction: GradientDirection,
) {
    if leds.is_empty() {
        return;
    }

    // Ensure proper ordering
    let (start_pos, end_pos, mut start_color, mut end_color) = if end_pos < start_pos {
        (end_pos, start_pos, end_color, start_color)
    } else {
        (start_pos, end_pos, start_color, end_color)
    };

    // Handle black/white edge cases for hue
    if end_color.val == 0 || end_color.sat == 0 {
        end_color.hue = start_color.hue;
    }
    if start_color.val == 0 || start_color.sat == 0 {
        start_color.hue = end_color.hue;
    }

    // Calculate distances in 8.7 fixed-point
    let sat_distance87 = (i16::from(end_color.sat) - i16::from(start_color.sat)) << 7;
    let val_distance87 = (i16::from(end_color.val) - i16::from(start_color.val)) << 7;

    let hue_delta = end_color.hue.wrapping_sub(start_color.hue);

    // Determine actual direction based on hue delta
    let actual_direction = match direction {
        GradientDirection::Shortest => {
            if hue_delta > 127 {
                GradientDirection::Backward
            } else {
                GradientDirection::Forward
            }
        }
        other => other,
    };

    let hue_distance87: i16 = if actual_direction == GradientDirection::Forward {
        i16::from(hue_delta) << 7
    } else {
        let backward_delta = 256u16.wrapping_sub(u16::from(hue_delta)) as u8;
        -((i16::from(backward_delta)) << 7)
    };

    let pixel_distance = end_pos.saturating_sub(start_pos);
    let divisor = if pixel_distance == 0 { 1 } else { pixel_distance as i32 };

    // Calculate 8.23 fixed-point deltas
    let hue_delta823 = ((i32::from(hue_distance87) * 65536) / divisor) * 2;
    let sat_delta823 = ((i32::from(sat_distance87) * 65536) / divisor) * 2;
    let val_delta823 = ((i32::from(val_distance87) * 65536) / divisor) * 2;

    // Initialize 8.24 accumulators
    let mut hue824 = u32::from(start_color.hue) << 24;
    let mut sat824 = u32::from(start_color.sat) << 24;
    let mut val824 = u32::from(start_color.val) << 24;

    let end_pos = end_pos.min(leds.len() - 1);
    for i in start_pos..=end_pos {
        leds[i] = hsv2rgb(Hsv {
            hue: (hue824 >> 24) as u8,
            sat: (sat824 >> 24) as u8,
            val: (val824 >> 24) as u8,
        });
        hue824 = hue824.wrapping_add(hue_delta823 as u32);
        sat824 = sat824.wrapping_add(sat_delta823 as u32);
        val824 = val824.wrapping_add(val_delta823 as u32);
    }
}

/// Fill three-color gradient using fixed-point math
fn fill_gradient_three_fp(leds: &mut [RGB<u8>], c1: Hsv, c2: Hsv, c3: Hsv) {
    if leds.is_empty() {
        return;
    }

    let len = leds.len();
    let half = len / 2;
    let last = len.saturating_sub(1);

    fill_gradient_fp(leds, 0, c1, half, c2, GradientDirection::Forward);
    if last > half {
        fill_gradient_fp(leds, half, c2, last, c3, GradientDirection::Forward);
    }
}

// =============================================================================
// Floating-point gradient helpers (for RainbowFlowEffect)
// =============================================================================

#[derive(Clone, Copy)]
enum HueDirection {
    Forward,
    Backward,
}

fn fill_gradient_three_float(
    leds: &mut [RGB<u8>],
    c1: Hsv,
    c2: Hsv,
    c3: Hsv,
    direction: HueDirection,
) {
    if leds.is_empty() {
        return;
    }

    let len = leds.len();
    let half = len / 2;
    let last = len - 1;

    fill_gradient_segment_float(leds, 0, half, c1, c2, direction);
    if last > half {
        fill_gradient_segment_float(leds, half, last, c2, c3, direction);
    }
}

#[allow(clippy::cast_precision_loss)]
fn fill_gradient_segment_float(
    leds: &mut [RGB<u8>],
    mut start_idx: usize,
    mut end_idx: usize,
    mut start_color: Hsv,
    mut end_color: Hsv,
    direction: HueDirection,
) {
    if leds.is_empty() {
        return;
    }

    if end_idx < start_idx {
        core::mem::swap(&mut start_idx, &mut end_idx);
        core::mem::swap(&mut start_color, &mut end_color);
    }

    end_idx = end_idx.min(leds.len() - 1);
    start_idx = min(start_idx, end_idx);

    if end_color.val == 0 || end_color.sat == 0 {
        end_color.hue = start_color.hue;
    }

    if start_color.val == 0 || start_color.sat == 0 {
        start_color.hue = end_color.hue;
    }

    let range = end_idx - start_idx;
    let hue_delta = hue_distance_float(start_color.hue, end_color.hue, direction);

    for step in 0..=range {
        let t = if range == 0 {
            0.0
        } else {
            step as f32 / range as f32
        };

        let hue = wrap_hue_float(start_color.hue, hue_delta, t);
        let sat = lerp_channel_float(start_color.sat, end_color.sat, t);
        let val = lerp_channel_float(start_color.val, end_color.val, t);

        leds[start_idx + step] = hsv2rgb(Hsv { hue, sat, val });
    }
}

#[allow(clippy::cast_lossless)]
fn hue_distance_float(start: u8, end: u8, direction: HueDirection) -> i16 {
    let start = i16::from(start);
    let end = i16::from(end);
    match direction {
        HueDirection::Forward => (end - start).rem_euclid(256),
        HueDirection::Backward => -(start - end).rem_euclid(256),
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
fn wrap_hue_float(start_hue: u8, delta: i16, t: f32) -> u8 {
    let offset = (f32::from(delta) * t) as i16;
    let value = i16::from(start_hue) + offset;
    value.rem_euclid(256) as u8
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_lossless)]
fn lerp_channel_float(start: u8, end: u8, t: f32) -> u8 {
    if start == end {
        return start;
    }
    let start_f = f32::from(start);
    let end_f = f32::from(end);
    let value = start_f + (end_f - start_f) * t;
    value.clamp(0.0, 255.0) as u8
}

// =============================================================================
// Common helpers
// =============================================================================

fn mirror_half(leds: &mut [RGB<u8>], center: usize) {
    if leds.is_empty() {
        return;
    }
    let count = leds.len();
    let limit = center.min(count);
    for i in 0..limit {
        let mirrored = count - 1 - i;
        leds[mirrored] = leds[i];
    }
}
