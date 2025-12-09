use heapless::Deque;

use crate::color::Rgb;
use crate::mode::ModeId;

/// Operations that can be performed on the light engine
///
/// N is the number of LEDs in the strip
#[derive(Debug, Clone, Copy)]
pub enum Operation {
    /// Set brightness
    SetBrightness(u8),
    /// Switch to a new mode with fade transition
    SwitchMode(ModeId),
    /// Update effect color
    SetColor(Rgb),
    /// Power off the light (fade out to 0, but preserve target brightness).
    PowerOff,
    /// Power on the light (fade in from 0 to the stored target brightness).
    PowerOn,
}

/// Stack of operations to be performed on the engine
///
/// N is the number of operations to store in the stack
#[derive(Debug, Default)]
pub struct OperationStack<const N: usize> {
    inner: Deque<Operation, N>,
    current: Option<Operation>,
}

impl<const N: usize> OperationStack<N> {
    pub const fn new() -> Self {
        Self {
            inner: Deque::new(),
            current: None,
        }
    }

    /// Push an operation onto the stack
    ///
    /// Returns the operation if the stack is full
    pub fn push(&mut self, operation: Operation) -> Result<(), Operation> {
        self.inner.push_back(operation)?;
        if self.current.is_none() {
            self.current = Some(operation);
        }
        Ok(())
    }

    /// Pop the current operation from the stack
    ///
    /// Returns None if the stack is empty
    pub fn pop(&mut self) -> Option<Operation> {
        self.current = self.inner.pop_front();
        self.current
    }

    /// Get the current operation from the stack
    ///
    /// Returns None if the stack is empty
    pub fn current(&self) -> Option<Operation> {
        self.current
    }

    /// Push a brightness operation onto the stack
    pub fn push_brightness(&mut self, brightness: u8) -> Result<(), Operation> {
        self.push(Operation::SetBrightness(brightness))
    }

    /// Push a color operation onto the stack
    pub fn push_color(&mut self, color: Rgb) -> Result<(), Operation> {
        self.push(Operation::SetColor(color))
    }

    /// Push a mode operation onto the stack
    pub fn push_mode(&mut self, mode: ModeId, brightness: u8) -> Result<(), Operation> {
        let free_slots = self.inner.capacity() - self.inner.len();
        let mode_op = Operation::SwitchMode(mode);
        if free_slots < 3 {
            return Err(mode_op);
        }
        self.push(Operation::SetBrightness(0))?;
        self.push(mode_op)?;
        self.push(Operation::SetBrightness(brightness))?;

        Ok(())
    }

    /// Push a power off operation onto the stack
    pub fn push_power_off(&mut self) -> Result<(), Operation> {
        self.push(Operation::PowerOff)
    }

    /// Push a power on operation onto the stack
    pub fn push_power_on(&mut self) -> Result<(), Operation> {
        self.push(Operation::PowerOn)
    }
}
