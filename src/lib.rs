#![warn(clippy::all)]

#[macro_use]
extern crate derive_more;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;


pub mod cameras;
pub mod db;
pub mod error;
mod locks;
pub mod prochost;
pub mod proxy;
pub mod settings;
pub mod stream;
pub mod ui;
pub mod users;


pub use error::*;
pub use locks::*;
