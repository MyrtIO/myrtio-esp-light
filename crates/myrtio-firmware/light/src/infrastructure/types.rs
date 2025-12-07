use core::cell::RefCell;

use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use myrtio_light_composer::{Command, CommandSender};

use crate::infrastructure::{config, drivers::EspLedDriver, repositories::LightNorFlashStorage};

pub(crate) type LightCommandSender = CommandSender<{ config::LIGHT_LED_COUNT }>;
pub(crate) type LightCommand = Command<{ config::LIGHT_LED_COUNT }>;
pub(crate) type LightDriver = EspLedDriver<'static, { config::LIGHT_LED_COUNT }>;
pub(crate) type LightStorageMutex = Mutex<CriticalSectionRawMutex, RefCell<LightNorFlashStorage>>;
