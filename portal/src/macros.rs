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


//#region Error Handling

/// Unwraps a result, but return early instead of panicking
macro_rules! unwrap_or_return {
    ($result:expr) => ({
        use log::error;
        match $result {
            Ok(result) => result,
            Err(err) => {
                error!("{}", err);
                return;
            }
        }
    })
}

//#endregion
