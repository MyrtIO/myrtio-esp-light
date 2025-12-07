#![no_std]

//! Light Engine v2 - State Machine Architecture
//!
//! Architecture layers:
//! - `driver` - Hardware abstraction (`[LedDriver]` trait + implementations)
//! - `effect` - Effect implementations and [`EffectSlot`] enum
//! - `processor` - Output processing (brightness, gamma, etc.)
//! - `transition` - Reusable transition utilities (color, etc.)
//! - `engine` - Main state machine orchestrator
//! - `state` - Shared state for external observation
//!
//! The engine is generic over `LedDriver`, allowing different hardware backends.

pub mod driver;
pub mod effect;
pub mod engine;
pub mod models;
pub mod processor;
pub mod state;
pub mod transition;
pub mod math8;

// Driver exports
pub use driver::LedDriver;

// Effect exports
pub use effect::EffectSlot;

// Engine exports
pub use engine::{
    Command, CommandChannel, CommandReceiver, CommandSender, EngineState, LightEngine,
    TransitionConfig,
};

// Processor exports
pub use processor::{ColorCorrection, OutputProcessor};

// State exports
pub use state::{EffectId, SharedState};

// Transition exports
pub use transition::ColorTransition;

pub use models::LightSnapshot;
