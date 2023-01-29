use std::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::{Arc, LockResult, Mutex, MutexGuard};

///
/// Thread-safe shared mutable ownership.
///
pub struct SharedMutable<T> {
    value: Arc<Mutex<T>>,
}

impl<T> Clone for SharedMutable<T> {
    fn clone(&self) -> Self {
        SharedMutable {
            value: self.value.clone(),
        }
    }
}


unsafe impl<T> Send for SharedMutable<T> {}

unsafe impl<T> Sync for SharedMutable<T> {}

impl<T: RefUnwindSafe + Sized> UnwindSafe for SharedMutable<T> {}


impl<T> SharedMutable<T> {
    pub fn new(value: T) -> Self {
        SharedMutable {
            value: Arc::new(Mutex::new(value)),
        }
    }

    ///
    /// Blocks until the lock is acquired.
    ///
    #[allow(dead_code)]
    pub fn lock_and_get(&self) -> LockResult<MutexGuard<'_, T>> {
        self.value.lock()
    }

    ///
    /// Try picking up the lock to see if it's locked.
    /// If accidentally acquired the lock, it will be unlocked immediately.
    ///
    #[allow(dead_code)]
    pub fn locked(&self) -> bool {
        match self.value.try_lock() {
            Ok(mutex_guard) => {
                drop(mutex_guard);
                false
            }
            Err(_) => true,
        }
    }
}
