use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait MutexExtensions<T> {
    /// A helper function used to unwrap a [`Mutex::lock`] [`LockResult`]
    /// without Clippy warning us.
    fn lock_and_unwrap(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexExtensions<T> for Mutex<T> {
    fn lock_and_unwrap(&self) -> MutexGuard<'_, T> {
        #[expect(
            clippy::unwrap_used,
            reason = "This is a helper to unwrap a LockResult so that there are no warnings \
                      everywhere."
        )]
        self.lock().unwrap()
    }
}

pub trait RwLockExtensions<T> {
    /// A helper function used to unwrap a [`RwLock::read`] [`LockResult`]
    /// without Clippy warning us.
    fn read_and_unwrap(&self) -> RwLockReadGuard<'_, T>;

    /// A helper function used to unwrap a [`RwLock::write`] [`LockResult`]
    /// without Clippy warning us.
    fn write_and_unwrap(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> RwLockExtensions<T> for RwLock<T> {
    fn read_and_unwrap(&self) -> RwLockReadGuard<'_, T> {
        #[expect(
            clippy::unwrap_used,
            reason = "This is a helper to unwrap a LockResult so that there are no warnings \
                      everywhere."
        )]
        self.read().unwrap()
    }

    fn write_and_unwrap(&self) -> RwLockWriteGuard<'_, T> {
        #[expect(
            clippy::unwrap_used,
            reason = "This is a helper to unwrap a LockResult so that there are no warnings \
                      everywhere."
        )]
        self.write().unwrap()
    }
}
