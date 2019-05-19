//! Configuration management


//#region Usings

use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufReader, Seek, SeekFrom};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::result;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use actix::{Actor, Addr, Arbiter, Context, Handler, Message};

use derive_more::Display;

use futures::future::Future;

use log::{error, warn};

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

//#endregion


//#region Error Handling

/// Error type returned by config operations
#[derive(Debug, Display)]
pub enum Error
{
    /// An I/O operation failed
    Io(io::Error),

    /// Serialization or deserialization of JSON failed
    Json(serde_json::Error),
}

impl From<io::Error> for Error
{
    fn from(err: io::Error) -> Self
    {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error
{
    fn from(err: serde_json::Error) -> Self
    {
        Error::Json(err)
    }
}

/// Result type returned by config operations
pub type Result<T> = result::Result<T, Error>;

//#endregion


//#region Utilities

/// Loads configuration from the given file
fn load_config<T>(file: &mut File) -> Result<T>
where T: DeserializeOwned
{
    file.seek(SeekFrom::Start(0))?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

/// Stores configuration to the given file
fn store_config<T>(config: &T, file: &mut File) -> Result<()>
where T: Serialize
{
    file.seek(SeekFrom::Start(0))?;
    file.set_len(0)?;
    serde_json::to_writer_pretty(&*file, config)?;
    file.sync_all()?;
    Ok(())
}

/// Retrieves a read or write guard from a potentially poisoned lock
///
/// A warning is logged if the lock is poisoned, but the guard is still returned.
macro_rules! rwl {
    ($lock:expr, $op:ident) => {
        $lock.$op()
            .unwrap_or_else(|err| {
                warn!("A configuration lock is poisoned");
                err.into_inner()
            })
    }
}

/// Retrieves an `RwLock`'s read guard
///
/// A warning is logged if the lock is poisoned, but the guard is still returned.
macro_rules! rwl_read {
    ($lock:expr) => (rwl!($lock, read))
}

/// Retrieves an `RwLock`'s write guard
///
/// A warning is logged if the lock is poisoned, but the guard is still returned.
macro_rules! rwl_write {
    ($lock:expr) => (rwl!($lock, write))
}

//#endregion


//#region System Configuration

// TODO: use regular config infrastructure to load and watch system configuration

/// Critical configuration needed for the system to operate correctly
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig
{
    /// Address on which LunaCam listens for HTTP requests
    pub listen: String,

    /// Path to static web content (e.g. stylesheets)
    pub static_path: String,

    /// Path to HTML templates
    pub template_path: String,

    /// Path to user configuration storage
    pub user_config_path: String,
}

impl SystemConfig
{
    /// Loads system configuration from the specified file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<SystemConfig>
    {
        load_config(&mut File::open(path)?)
    }
}

//#endregion


//#region Configuration Flusher

/// Performs asynchronous flushing of config data to the filesystem
struct ConfigFlusher<T>
{
    config: Arc<RwLock<T>>,
    file: File,
}

impl<T> Actor for ConfigFlusher<T>
where T: 'static
{
    type Context = Context<Self>;
}

/// Signals a `ConfigFlusher` to flush configuration
struct Flush;

impl Message for Flush
{
    type Result = Result<()>;
}

impl<T> Handler<Flush> for ConfigFlusher<T>
where T: Serialize + 'static
{
    type Result = Result<()>;

    fn handle(&mut self, _: Flush, _: &mut Context<Self>) -> Self::Result
    {
        let config = rwl_read!(self.config);
        store_config(&*config, &mut self.file)
    }
}

//#endregion


//#region Write Guard

/// RAII structure used to ensure configuration is flushed after it is modified
pub struct ConfigWriteGuard<'a, T>
where T: Serialize + 'static
{
    inner: RwLockWriteGuard<'a, T>,
    flusher: &'a Addr<ConfigFlusher<T>>,
}

impl<'a, T> Deref for ConfigWriteGuard<'a, T>
where T: Serialize
{
    type Target = T;

    fn deref(&self) -> &T
    {
        self.inner.deref()
    }
}

impl<'a, T> DerefMut for ConfigWriteGuard<'a, T>
where T: Serialize
{
    fn deref_mut(&mut self) -> &mut T
    {
        self.inner.deref_mut()
    }
}

impl<'a, T> Drop for ConfigWriteGuard<'a, T>
where T: Serialize
{
    fn drop(&mut self)
    {
        Arbiter::spawn(
            self.flusher.send(Flush)
                .map_err(|err|
                    error!("Failed to communicate with configuration flusher: {}", err)
                )
                .map(|res| res.unwrap_or_else(|err|
                    error!("Failed to flush configuration: {}", err))
                )
        );
    }
}

//#endregion


//#region Configuration Management

// TODO: Watch backing file for changes
// TODO: Support read-only config, use for system parameters

/// Manages access to filesystem-backed configuration
///
/// `Config` provides shared access to an instance of `T` with very similar semantics to `RwLock`.
/// Additionally, any changes made to the instance are serialized and flushed to a file.
pub struct Config<T>
where T: 'static
{
    config: Arc<RwLock<T>>,
    flusher: Addr<ConfigFlusher<T>>,
}

impl<T> Config<T>
where T: Default + DeserializeOwned + Serialize + 'static
{
    /// Creates a new `Config` with the given name
    ///
    /// `name` is used as name of the file backing the configuration, so it must be unique across
    /// all instances of `Config`.
    pub fn new(name: &str, sys: &SystemConfig) -> Result<Self>
    {
        // Open backing file, creating it if it does not exist
        // TODO: this method should probably just accept a path instead of trying to construct one
        let mut path = PathBuf::from(&sys.user_config_path);
        path.push(format!("{}.json", name));
        let already_exists = path.exists();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        // If file was already present, load its contents
        let config = if already_exists {
            load_config(&mut file)?
        } else {
            Default::default()
        };

        // If file is new, write default contents
        if !already_exists {
            store_config(&config, &mut file)?;
        }

        let config = Arc::new(RwLock::new(config));
        let flusher = ConfigFlusher {
            config: config.clone(),
            file: file,
        };

        Ok(Config {
            config: config,
            flusher: flusher.start(),
        })
    }

    /// Locks configuration for shared read access
    ///
    /// Current thread is blocked until lock can be acquired.
    pub fn read(&self) -> RwLockReadGuard<T>
    {
        rwl_read!(self.config)
    }

    /// Locks configuration for exclusive write access
    ///
    /// Current thread is blocked until lock can be acquired.
    pub fn write(&self) -> ConfigWriteGuard<T>
    {
        ConfigWriteGuard {
            inner: rwl_write!(self.config),
            flusher: &self.flusher,
        }
    }
}

/// Manual implementation of `Clone`
///
/// Needed because `T` need not implement `Clone`. See also
/// [RFC 2353](https://github.com/rust-lang/rfcs/pull/2353)
impl<T> Clone for Config<T>
where T: Default + DeserializeOwned + Serialize + 'static
{
    fn clone(&self) -> Self
    {
        Config {
            config: self.config.clone(),
            flusher: self.flusher.clone()
        }
    }
}

//#endregion
