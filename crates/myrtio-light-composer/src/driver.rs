//! LED Driver abstraction layer
//!
//! Provides a trait-based abstraction for LED strip drivers,
//! allowing the light engine to be hardware-agnostic.

use smart_leds::RGB;

/// Abstract LED driver trait
///
/// Implement this trait to support different hardware platforms.
/// The light engine is generic over this trait.
pub trait LedDriver<const N: usize> {
    /// Write colors to the LED strip
    fn write(&mut self, colors: &[RGB<u8>; N]);
}
