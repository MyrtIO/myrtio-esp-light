use core::cell::RefCell;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::domain::types::LightUsecasesPortRef;

pub(crate) static LIGHT_USECASES: Mutex<
    CriticalSectionRawMutex,
    RefCell<Option<LightUsecasesPortRef>>,
> = Mutex::new(RefCell::new(None));
