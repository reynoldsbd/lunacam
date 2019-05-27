//! Utility macros


//#region RwLock Wrappers

/// Retrieves a read or write guard from a potentially poisoned RwLock
///
/// A warning is logged if the lock is poisoned, but the guard is still returned.
macro_rules! _rwl {
    ($lock:expr, $op:ident) => ({
        use log::warn;
        $lock.$op()
            .unwrap_or_else(|err| {
                warn!("A lock is poisoned");
                err.into_inner()
            })
    })
}

/// Retrieves an `RwLock`'s read guard
///
/// A warning is logged if the lock is poisoned, but the guard is still returned.
macro_rules! rwl_read {
    ($lock:expr) => (_rwl!($lock, read))
}

/// Retrieves an `RwLock`'s write guard
///
/// A warning is logged if the lock is poisoned, but the guard is still returned.
macro_rules! rwl_write {
    ($lock:expr) => (_rwl!($lock, write))
}

//#endregion
