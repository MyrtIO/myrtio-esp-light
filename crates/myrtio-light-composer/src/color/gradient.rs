use smart_leds::hsv::hsv2rgb;

use crate::color::{Hsv, Rgb};

/// Hue direction for gradient calculation
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GradientDirection {
    Forward,
    Backward,
    Shortest,
}

/// Fill gradient using fixed-point 8.24 arithmetic (ported from `FastLED`)
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_possible_wrap
)]
pub fn fill_gradient_fp(
    leds: &mut [Rgb],
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
    let divisor = if pixel_distance == 0 {
        1
    } else {
        pixel_distance as i32
    };

    // Calculate 8.23 fixed-point deltas
    let hue_delta823 = ((i32::from(hue_distance87) * 65536) / divisor) * 2;
    let sat_delta823 = ((i32::from(sat_distance87) * 65536) / divisor) * 2;
    let val_delta823 = ((i32::from(val_distance87) * 65536) / divisor) * 2;

    // Initialize 8.24 accumulators
    let mut hue824 = u32::from(start_color.hue) << 24;
    let mut sat824 = u32::from(start_color.sat) << 24;
    let mut val824 = u32::from(start_color.val) << 24;

    let end_pos = end_pos.min(leds.len() - 1);
    for led in leds.iter_mut().take(end_pos + 1).skip(start_pos) {
        *led = hsv2rgb(Hsv {
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
pub fn fill_gradient_three_fp(leds: &mut [Rgb], c1: Hsv, c2: Hsv, c3: Hsv) {
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

// /// Hue direction for gradient calculation
// #[derive(Clone, Copy)]
// enum HueDirection {
//     Forward,
//     Backward,
// }

// /// Fill three-color gradient using floating-point math
// pub fn fill_gradient_three_float(
//     leds: &mut [Rgb],
//     c1: Hsv,
//     c2: Hsv,
//     c3: Hsv,
//     direction: HueDirection,
// ) {
//     if leds.is_empty() {
//         return;
//     }

//     let len = leds.len();
//     let half = len / 2;
//     let last = len - 1;

//     fill_gradient_segment_float(leds, 0, half, c1, c2, direction);
//     if last > half {
//         fill_gradient_segment_float(leds, half, last, c2, c3, direction);
//     }
// }

// /// Fill gradient segment using floating-point math
// #[allow(clippy::cast_precision_loss)]
// pub fn fill_gradient_segment_float(
//     leds: &mut [Rgb],
//     mut start_idx: usize,
//     mut end_idx: usize,
//     mut start_color: Hsv,
//     mut end_color: Hsv,
//     direction: HueDirection,
// ) {
//     if leds.is_empty() {
//         return;
//     }

//     if end_idx < start_idx {
//         core::mem::swap(&mut start_idx, &mut end_idx);
//         core::mem::swap(&mut start_color, &mut end_color);
//     }

//     end_idx = end_idx.min(leds.len() - 1);
//     start_idx = min(start_idx, end_idx);

//     if end_color.val == 0 || end_color.sat == 0 {
//         end_color.hue = start_color.hue;
//     }

//     if start_color.val == 0 || start_color.sat == 0 {
//         start_color.hue = end_color.hue;
//     }

//     let range = end_idx - start_idx;
//     let hue_delta = hue_distance_float(start_color.hue, end_color.hue, direction);

//     for step in 0..=range {
//         let t = if range == 0 {
//             0.0
//         } else {
//             step as f32 / range as f32
//         };

//         let hue = wrap_hue_float(start_color.hue, hue_delta, t);
//         let sat = lerp_channel_float(start_color.sat, end_color.sat, t);
//         let val = lerp_channel_float(start_color.val, end_color.val, t);

//         leds[start_idx + step] = hsv2rgb(Hsv { hue, sat, val });
//     }
// }

// #[allow(clippy::cast_lossless)]
// fn hue_distance_float(start: u8, end: u8, direction: HueDirection) -> i16 {
//     let start = i16::from(start);
//     let end = i16::from(end);
//     match direction {
//         HueDirection::Forward => (end - start).rem_euclid(256),
//         HueDirection::Backward => -(start - end).rem_euclid(256),
//     }
// }

// #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
// fn wrap_hue_float(start_hue: u8, delta: i16, t: f32) -> u8 {
//     let offset = (f32::from(delta) * t) as i16;
//     let value = i16::from(start_hue) + offset;
//     value.rem_euclid(256) as u8
// }

// #[allow(
//     clippy::cast_possible_truncation,
//     clippy::cast_sign_loss,
//     clippy::cast_lossless
// )]
// fn lerp_channel_float(start: u8, end: u8, t: f32) -> u8 {
//     if start == end {
//         return start;
//     }
//     let start_f = f32::from(start);
//     let end_f = f32::from(end);
//     let value = start_f + (end_f - start_f) * t;
//     value.clamp(0.0, 255.0) as u8
// }
