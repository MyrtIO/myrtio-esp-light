use core::cell::RefCell;

use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};

use crate::infrastructure::{drivers::EspLedDriver, repositories::LightNorFlashStorage};

pub(crate) type LightDriver = EspLedDriver<'static>;
pub(crate) type LightStorageMutex = Mutex<CriticalSectionRawMutex, RefCell<LightNorFlashStorage>>;
