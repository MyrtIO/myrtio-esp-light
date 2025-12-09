use embassy_time::Duration;

/// Scale an 8-bit value by a factor (0-255 = 0.0-1.0)
///
/// Uses integer math for efficiency on embedded systems.
#[inline]
#[allow(clippy::cast_lossless)]
pub const fn scale8(value: u8, scale: u8) -> u8 {
    ((value as u16 * (1 + scale as u16)) >> 8) as u8
}

/// Blend two 8-bit values
#[inline]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub const fn blend8(a: u8, b: u8, amount_of_b: u8) -> u8 {
    let delta = b as i16 - a as i16;

    let mut partial: u32 = (a as u32) << 16; // a * 65536
    partial = partial.wrapping_add(
        (delta as u32)
            .wrapping_mul(amount_of_b as u32)
            .wrapping_mul(257),
    ); // (b - a) * amount_of_b * 257
    partial = partial.wrapping_add(0x8000); // + 32768 for rounding

    (partial >> 16) as u8
}

/// Calculate progress (0-255) based on elapsed time and duration
///
///
#[allow(clippy::cast_possible_truncation)]
#[inline]
pub const fn progress8(elapsed: Duration, duration: Duration) -> u8 {
    if duration.as_millis() == 0 {
        return 0;
    }
    if elapsed.as_millis() >= duration.as_millis() {
        return 255;
    }

    ((elapsed.as_millis() * 255) / duration.as_millis()) as u8
}
