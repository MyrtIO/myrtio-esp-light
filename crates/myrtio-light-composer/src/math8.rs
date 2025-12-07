use embassy_time::Duration;

/// Scale an 8-bit value by a factor (0-255 = 0.0-1.0)
///
/// Uses integer math for efficiency on embedded systems.
#[inline]
#[allow(clippy::cast_lossless)]
pub fn scale8(value: u8, scale: u8) -> u8 {
    ((value as u16 * scale as u16) >> 8) as u8
}

/// Blend two 8-bit values
///
/// # Arguments
/// * `a` - First value
/// * `b` - Second value  
/// * `amount_of_b` - Blend factor (0 = all a, 255 = all b)
#[inline]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn blend8(a: u8, b: u8, amount_of_b: u8) -> u8 {
    // Fast integer blend: a + (b - a) * amount / 256
    let a = i16::from(a);
    let b = i16::from(b);
    let amount = i16::from(amount_of_b);

    (a + (((b - a) * amount) >> 8)) as u8
}

/// Calculate progress (0-255) based on elapsed time and duration
///
/// # Arguments
/// * `elapsed` - Elapsed time
/// * `duration` - Total duration
///
/// # Returns
/// * `progress` - Progress (0-255)
///
#[allow(clippy::cast_possible_truncation)]
#[inline]
pub fn progress8(elapsed: Duration, duration: Duration) -> u8 {
    if duration.as_millis() == 0 {
        return 0;
    }
    if elapsed.as_millis() >= duration.as_millis() {
        return 255;
    }

    ((elapsed.as_millis() * 255) / duration.as_millis()) as u8
}
