#![no_std]
#![no_main]

pub mod color;
pub mod command;
pub mod effect;
pub mod engine;
pub mod gamma;
pub mod math8;
pub mod mode;
pub mod operation;
pub mod transition;

pub use command::{Command, CommandChannel, CommandReceiver, CommandSender};
pub use effect::EffectProcessorConfig;
pub use engine::{LightEngine, LightEngineConfig, TransitionTimings};
pub use gamma::ws2812_lut;
pub use mode::{ModeId, ModeSlot};
pub use operation::{Operation, OperationStack};

pub use color::{Hsv, Rgb};
pub use math8::{ease_in_out_quad, U8Adjuster};

/// Abstract LED driver trait
///
/// Implement this trait to support different hardware platforms.
/// The light engine is generic over this trait.
pub trait LedDriver {
    /// Write colors to the LED strip
    fn write<const N: usize>(&mut self, colors: &[Rgb; N]);
}
