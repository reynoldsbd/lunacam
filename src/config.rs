//! Configuration management


//#region Usings

use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::result;

use base64::STANDARD;

use base64_serde::base64_serde_type;

use derive_more::Display;

use serde::Deserialize;

//#endregion


//#region Error Handling

/// Error type returned by config operations
#[derive(Debug, Display)]
pub enum Error {
    Io(io::Error),
    Json(serde_json::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

/// Result type returned by config operations
pub type Result<T> = result::Result<T, Error>;

//#endregion


//#region System Configuration

/// Provides base64 encoding/decoding for the configuration system
base64_serde_type!(BASE64, STANDARD);

/// Critical configuration needed for the system to operate correctly
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig {

    /// Address on which LunaCam listens for HTTP requests
    pub listen: String,

    /// Path to static web content (e.g. stylesheets)
    pub static_path: String,

    /// Path to HTML templates
    pub template_path: String,

    /// Path to user configuration storage
    pub user_config_path: String,

    // TODO: move these into user configs
    pub admin_password: String,
    #[serde(with = "BASE64")]
    pub secret: Vec<u8>,
    pub user_password: String,
}

impl SystemConfig {

    /// Loads system configuration from the specified file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<SystemConfig> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}

//#endregion
