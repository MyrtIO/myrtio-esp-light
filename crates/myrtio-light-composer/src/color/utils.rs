pub use smart_leds::hsv::hsv2rgb;

use crate::{color::Rgb, math8::blend8};

/// Mirror the first half of the array around the center
pub fn mirror_half(leds: &mut [Rgb]) {
    if leds.is_empty() {
        return;
    }
    // Compute center for mirroring
    let leds_len = leds.len();
    let mut center = leds_len / 2;
    if !leds_len.is_multiple_of(2) {
        center += 1;
    }
    center = center.min(leds_len);
    // Mirror the first half of the array around the center
    for i in 0..center {
        let mirrored = leds_len - 1 - i;
        leds[mirrored] = leds[i];
    }
}

/// Blend two RGB colors
///
/// # Arguments
/// * `a` - First color
/// * `b` - Second color
/// * `amount_of_b` - Blend factor (0 = all a, 255 = all b)
#[inline]
pub fn blend_colors(a: Rgb, b: Rgb, amount_of_b: u8) -> Rgb {
    Rgb {
        r: blend8(a.r, b.r, amount_of_b),
        g: blend8(a.g, b.g, amount_of_b),
        b: blend8(a.b, b.b, amount_of_b),
    }
}

/// Create an RGB color from a u32 value (0xRRGGBB format)
pub const fn rgb_from_u32(color: u32) -> Rgb {
    Rgb {
        r: ((color >> 16) & 0xFF) as u8,
        g: ((color >> 8) & 0xFF) as u8,
        b: (color & 0xFF) as u8,
    }
}
