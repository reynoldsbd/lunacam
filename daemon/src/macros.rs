//! Utility macros


/// Unwraps a result, but return early instead of panicking
// macro_rules! unwrap_or_return {
//     ($result:expr) => ({
//         use log::error;
//         match $result {
//             Ok(result) => result,
//             Err(err) => {
//                 error!("{}", err);
//                 return;
//             }
//         }
//     })
// }


macro_rules! lock_unwrap {
    ($lock:expr) => ({
        use log::warn;
        $lock.lock()
            .unwrap_or_else(|err| {
                warn!("A lock is poisoned");
                err.into_inner()
            })
    })
}
