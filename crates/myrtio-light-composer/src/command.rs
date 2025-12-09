use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};

use crate::color::Rgb;
use crate::mode::ModeId;

/// Operations that can be performed on the light engine
///
/// N is the number of LEDs in the strip
#[derive(Debug, Clone, Copy)]
pub enum Command {
    /// Set brightness
    SetBrightness(u8),
    /// Switch to a new mode with fade transition
    SwitchMode(ModeId),
    /// Update effect color
    SetColor(Rgb),
    /// Set color temperature
    SetColorTemperature(u16),
    /// Power off the light (fade out to 0, but preserve target brightness).
    PowerOff,
    /// Power on the light (fade in from 0 to the stored target brightness).
    PowerOn,
}

const COMMAND_CHANNEL_SIZE: usize = 4;

/// Type alias for command sender
pub type CommandSender = Sender<'static, CriticalSectionRawMutex, Command, COMMAND_CHANNEL_SIZE>;

/// Type alias for command receiver  
pub type CommandReceiver =
    Receiver<'static, CriticalSectionRawMutex, Command, COMMAND_CHANNEL_SIZE>;

/// Type alias for the command channel
pub type CommandChannel = Channel<CriticalSectionRawMutex, Command, COMMAND_CHANNEL_SIZE>;
