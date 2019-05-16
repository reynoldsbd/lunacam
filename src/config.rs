//! Configuration management


//#region Usings

use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufReader, Seek, SeekFrom};
use std::marker::PhantomData;
use std::mem;
use std::path::{Path, PathBuf};
use std::result;

use actix::{Actor, Context, Handler, Message};

use base64::STANDARD;

use base64_serde::base64_serde_type;

use derive_more::Display;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

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


//#region User Configuration

// TODO: Hide actor behind some opaque object, allowing for "load" operation without message passing

/// Dynamic configuration that can be modified by the user
pub trait UserConfig: Clone + Default + DeserializeOwned + Serialize + 'static {}

/// Loads configuration from the given file
fn load_config<T: UserConfig>(file: &mut File) -> Result<T> {
    file.seek(SeekFrom::Start(0))?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

/// Stores configuration to the given file
fn store_config<T: UserConfig>(config: &T, file: &mut File) -> Result<()> {
    file.seek(SeekFrom::Start(0))?;
    file.set_len(0)?;
    serde_json::to_writer_pretty(&*file, config)?;
    file.sync_all()?;
    Ok(())
}

/// Provides access to filesystem-backed configuration
pub struct Config<T> {
    file: File,
    config: T,
}

impl<T: UserConfig> Config<T> {

    /// Creates a new `Config` with the given name
    ///
    /// `name` is used as name of the file backing the configuration, so it must be unique across
    /// all instances of `Config`.
    pub fn new(name: &str, sys: &SystemConfig) -> Result<Self> {

        // Open backing file, creating it if it does not exist
        let mut path = PathBuf::from(&sys.user_config_path);
        path.push(format!("{}.json", name));
        let existed = path.exists();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        // If file was already present, load its contents
        let config = if existed {
            load_config(&mut file)?
        } else {
            Default::default()
        };

        // If file is new, write default contents
        if !existed {
            store_config(&config, &mut file)?;
        }

        Ok(Config {
            file: file,
            config: config,
        })
    }
}

impl<T: UserConfig> Actor for Config<T> {
    type Context = Context<Self>;
}

/// Loads configuration from a `Config`
pub struct LoadConfig<T>(PhantomData<T>);

impl<T> LoadConfig<T> {

    /// Creates a new `LoadConfig` message
    pub fn new() -> Self {
        LoadConfig(PhantomData)
    }
}

impl<T: UserConfig> Message for LoadConfig<T> {
    type Result = Result<T>;
}

impl<T: UserConfig> Handler<LoadConfig<T>> for Config<T> {
    type Result = Result<T>;

    fn handle(&mut self, _: LoadConfig<T>, _: &mut Context<Self>) -> Self::Result {
        Ok(self.config.clone())
    }
}

/// Stores configuration to a `Config`
pub struct StoreConfig<T>(T);

impl<T: UserConfig> Message for StoreConfig<T> {
    type Result = Result<()>;
}

impl<T: UserConfig> Handler<StoreConfig<T>> for Config<T> {
    type Result = Result<()>;

    fn handle(&mut self, store: StoreConfig<T>, _: &mut Context<Self>) -> Self::Result {
        mem::replace(&mut self.config, store.0);
        store_config(&self.config, &mut self.file)?;
        Ok(())
    }
}

//#endregion
