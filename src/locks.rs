//! Macros for working with locks


/// Unconditionally retrieves a lock guard
///
/// Clients should prefer to use `do_lock`, `do_read`, or `do_write`.
#[macro_export]
macro_rules! _unwrap_lock {
    ($lock:expr, $op:ident) => ({
        use log::warn;
        $lock.$op()
            .unwrap_or_else(|err| {
                warn!("a lock is poisoned");
                err.into_inner()
            })
    })
}


/// Unconditionally retrieves the guard from a `Mutex`
///
/// A warning is emitted if the `Mutex` is poisoned.
#[macro_export]
macro_rules! do_lock {
    ($lock:expr) => ({
        use $crate::_unwrap_lock;
        _unwrap_lock!($lock, lock)
    })
}


/// Unconditionally retrieves a read guard from a `RwLock`
///
/// A warning is emitted if the `RwLock` is poisoned.
#[macro_export]
macro_rules! do_read {
    ($lock:expr) => ({
        use $crate::_unwrap_lock;
        _unwrap_lock!($lock, read)
    })
}


/// Unconditionally retrieves the write guard from a `RwLock`
///
/// A warning is emitted if the `RwLock` is poisoned.
#[macro_export]
macro_rules! do_write {
    ($lock:expr) => ({
        use $crate::_unwrap_lock;
        _unwrap_lock!($lock, write)
    })
}
