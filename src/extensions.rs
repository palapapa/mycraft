use std::sync::*;

pub trait MutexExtensions<'a, T> {
    fn lock_and_unwrap(&'a self) -> MutexGuard<'a, T>;
}

impl<'a, T> MutexExtensions<'a, T> for Mutex<T> {
    /// A helper function used to unwrap a [`Mutex`] without Clippy warning us.
    fn lock_and_unwrap(&'a self) -> MutexGuard<'a, T> {
        #[expect(clippy::unwrap_used, reason = "This is a helper to unwrap a LockResult so that there are no warnings everywhere.")]
        self.lock().unwrap()
    }
}
