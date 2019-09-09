//! Types used by the LunaCam web API

#[macro_use]
extern crate diesel;

use serde::{Deserialize, Serialize};

mod orientation;
pub use orientation::Orientation;


/// Video stream settings
#[derive(Clone, Copy, Default)]
#[derive(Deserialize, Serialize)]
pub struct StreamSettings {
    pub enabled: Option<bool>,
    pub orientation: Option<Orientation>,
}


/// Streaming camera settings
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraSettings {
    pub enabled: Option<bool>,
    pub hostname: Option<String>,
    pub id: Option<i32>,
    pub device_key: Option<String>,
    pub friendly_name: Option<String>,
    pub orientation: Option<Orientation>,
}
