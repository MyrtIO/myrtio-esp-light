#![no_std]
#![no_main]

pub mod color;
pub mod command;
pub mod effect;
pub mod engine;
pub mod math8;
pub mod mode;
pub mod operation;
pub mod transition;

pub use command::{Command, CommandChannel, CommandReceiver, CommandSender};
pub use effect::EffectProcessorConfig;
pub use engine::{LightEngine, LightEngineConfig, TransitionConfig};
pub use mode::{ModeId, ModeSlot};
pub use operation::{Operation, OperationStack};

pub use color::{Rgb, Hsv};

/// Abstract LED driver trait
///
/// Implement this trait to support different hardware platforms.
/// The light engine is generic over this trait.
pub trait LedDriver {
    /// Write colors to the LED strip
    fn write<const N: usize>(&mut self, colors: &[Rgb; N]);
}
