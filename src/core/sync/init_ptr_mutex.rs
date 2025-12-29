use core::ops::{Deref, DerefMut};

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    mutex::{Mutex, MutexGuard, TryLockError},
};
use static_cell::StaticCell;

/// Async "init-once" mutex for storing a raw pointer to some `'static` value.
///
/// This is a small helper to avoid `Mutex<RefCell<Option<T>>>` + `unwrap()` at
/// call sites. It is especially handy for storing trait objects:
/// `InitPtrMutex<dyn Trait>`.
///
/// # Safety / Invariants
/// - You must initialize it exactly once via [`init`] / [`init_ptr`].
/// - The pointer must remain valid for the entire program lifetime.
/// - Mutable access is serialized by the mutex guard.
pub(crate) struct InitPtrMutex<T: ?Sized> {
    inner: Mutex<CriticalSectionRawMutex, StaticCell<Option<*mut T>>>,
}

impl<T: ?Sized> InitPtrMutex<T> {
    pub(crate) const fn new() -> Self {
        Self {
            inner: Mutex::new(StaticCell::new()),
        }
    }

    pub(crate) async fn init(&'static self, value: &'static mut T) {
        self.init_ptr(core::ptr::from_mut(value)).await;
    }

    pub(crate) async fn init_ptr(&'static self, ptr: *mut T) {
        let guard = self.inner.lock().await;
        guard.init(Some(ptr));
    }

    pub(crate) async fn lock(&'static self) -> InitPtrLock<'static, T> {
        InitPtrLock {
            guard: self.inner.lock().await,
        }
    }

    pub(crate) fn try_lock(&'static self) -> Result<InitPtrLock<'static, T>, TryLockError> {
        self.inner
            .try_lock()
            .map(|guard| InitPtrLock { guard })
    }
}

pub(crate) struct InitPtrLock<'a, T: ?Sized> {
    guard: MutexGuard<'a, CriticalSectionRawMutex, Option<*mut T>>,
}

impl<T: ?Sized> Deref for InitPtrLock<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let ptr = self
            .guard
            .as_ref()
            .expect("InitPtrMutex is not initialized");
        unsafe { &**ptr }
    }
}

impl<T: ?Sized> DerefMut for InitPtrLock<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let ptr = self
            .guard
            .as_mut()
            .expect("InitPtrMutex is not initialized");
        unsafe { &mut **ptr }
    }
}

unsafe impl<T: ?Sized> Send for InitPtrLock<'_, T> {}
unsafe impl<T: ?Sized> Sync for InitPtrLock<'_, T> {}