//! Common utilities used by LunaCam services

#[macro_use]
extern crate derive_more;

mod lock_macros;
mod result;

pub use lock_macros::*;
pub use result::*;
