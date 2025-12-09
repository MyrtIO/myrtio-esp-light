use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};

use crate::operation::Operation;

/// Commands that can be sent to the light engine
pub type Command = Operation;

const COMMAND_CHANNEL_SIZE: usize = 4;

/// Type alias for command sender
pub type CommandSender = Sender<'static, CriticalSectionRawMutex, Command, COMMAND_CHANNEL_SIZE>;

/// Type alias for command receiver  
pub type CommandReceiver =
    Receiver<'static, CriticalSectionRawMutex, Command, COMMAND_CHANNEL_SIZE>;

/// Type alias for the command channel
pub type CommandChannel = Channel<CriticalSectionRawMutex, Command, COMMAND_CHANNEL_SIZE>;
