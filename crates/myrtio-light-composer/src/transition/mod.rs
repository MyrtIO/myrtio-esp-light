//! Transition utilities for smooth value changes
//!
//! This module provides reusable transition components that effects
//! and other parts of the light engine can use for smooth animations.

mod color;

pub use color::{ColorTransition, blend_colors};
