//! Manages video stream transcoding
//!
//! An external program (FFmpeg) is used to capture raw camera frames and transcode them into a
//! suitable streaming format. This module provides an API for managing the behavior and lifecycle
//! of the transcoding process.

use std::io;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use diesel::sqlite::SqliteConnection;
use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use tokio::executor::{Executor, SpawnError};
use tokio::prelude::*;
use tokio::sync::oneshot::{self, Sender};
use tokio::timer::{Interval};
use crate::{Orientation};
use crate::settings::{self, SettingsError};


//#region Error Handling

/// Error type returned by transcoder APIs
#[derive(Debug)]
pub enum TranscoderError {
    Io(io::Error),
    Settings(SettingsError),
    Tokio(SpawnError),
    Uninitialized,
    Watchdog,
}

impl From<io::Error> for TranscoderError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<SettingsError> for TranscoderError {
    fn from(err: SettingsError) -> Self {
        Self::Settings(err)
    }
}

impl From<SpawnError> for TranscoderError {
    fn from(err: SpawnError) -> Self {
        Self::Tokio(err)
    }
}

/// Result type returned by transcoder APIs
pub type Result<T> = std::result::Result<T, TranscoderError>;

//#endregion


//#region Transcoder Process Host

/// State of the transcoding system
#[derive(Clone, Copy, Default)]
#[derive(Deserialize, Serialize)]
pub struct TranscoderState {
    pub enabled: bool,
    pub orientation: Orientation,
}

/// Hosts and monitors the transcoder process
#[derive(Default)]
struct Host {
    status: TranscoderState,
    db_conn: Option<SqliteConnection>,
    tc_proc: Option<Child>,
    wdg_chan: Option<Sender<()>>,
    exec: Option<Box<dyn Executor + Send>>,
}


fn start_transcoder(host: &mut Host) -> Result<()> {

    assert!(host.tc_proc.is_none());

    debug!("starting transcoder process");
    let mut cmd = Command::new("ffmpeg");
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args(&[
            // General configuration
            "-hide_banner",
            "-loglevel", "error",

            // Input stream
            "-f", "v4l2",
            "-input_format", "h264",
            "-video_size", "1280x720",
            "-i", "/dev/video0",

            // Operation
            "-c:v", "copy",
            "-c:a", "aac",

            // Output stream
            "-f", "hls",
            "-hls_flags", "delete_segments",
            "/tmp/lunacam/hls/video0.m3u8",
        ]);
    host.tc_proc = Some(cmd.spawn()?);

    Ok(())
}


fn watchdog_tick() {

    trace!("watchdog tick");

    let mut host = lock_unwrap!(HOST);

    assert!(host.tc_proc.is_some(), "transcoder is not running");
    assert!(host.wdg_chan.is_some(), "watchdog is not running");
    assert!(host.exec.is_some(), "transcoder is not initialized");

    let res = host.tc_proc.as_mut()
        .unwrap()
        .try_wait();

    match res {

        // Everything is OK
        Ok(None) => (),

        // Child process exited unexpectedly
        Ok(Some(status)) => {
            warn!("Transcoder exited unexpectedly (status {}). Restarting...", status);
            host.tc_proc.take();
            start_transcoder(&mut host)
                .expect("watchdog failed to restart transcoder");
        },

        // Unexpected error
        Err(err) => error!("Error checking transcoder status: {}", err),
    }
}


/// Watchdog tick interval
const WDG_INTERVAL: u64 = 2;


fn start_host(host: &mut Host) -> Result<()> {

    assert!(host.wdg_chan.is_none());

    start_transcoder(host)?;

    trace!("starting watchdog");
    let (tx, mut rx) = oneshot::channel();
    let task = Interval::new(Instant::now(), Duration::from_secs(WDG_INTERVAL))
        .take_while(move |_| {
            let res = rx.poll().expect("watchdog failed to poll channel");
            Ok(res == Async::NotReady)
        })
        .for_each(|_| Ok(watchdog_tick()))
        .map_err(|e| {
            error!("Watchdog encountered unexpected timer error: {}", e);
        });
    host.exec.as_mut()
        .ok_or(TranscoderError::Uninitialized)?
        .spawn(Box::new(task))?;
    host.wdg_chan = Some(tx);

    Ok(())
}


fn stop_host(host: &mut Host) -> Result<()> {

    assert!(host.tc_proc.is_some(), "transcoder is not running");
    assert!(host.wdg_chan.is_some(), "watchdog is not running");

    trace!("stopping watchdog");
    host.wdg_chan.take()
        .unwrap()
        .send(())
        .map_err(|_| TranscoderError::Watchdog)?;

    debug!("stopping transcoder process");
    host.tc_proc.take()
        .unwrap()
        .kill()?;

    Ok(())
}

//#endregion


//#region Transcoder API

lazy_static! {
    static ref HOST: Mutex<Host> = Default::default();
}


const TC_STATUS_SETTING: &str = "transcoderStatus";


/// Initializes the transcoder
///
/// Loads state from persistent storage and initializes the transcoder according to that state. This
/// function must be called exactly once prior to using any other APIs from this module.
///
/// `exec` is used to spawn a watchdog which monitors and restarts the child transcoding process.
///
/// # Panics
///
/// Panics if called more than once.
pub fn initialize<T>(exec: T, conn: SqliteConnection) -> Result<()>
where T: Executor + Send + 'static
{
    let mut host = lock_unwrap!(HOST);

    assert!(host.db_conn.is_none(), "multiple calls to transcoder::initialize");

    trace!("initializing transcoder");
    if let Some(status) = settings::get(TC_STATUS_SETTING, &conn)? {
        host.status = status;
    } else {
        trace!("using default transcoder settings");
    }
    host.db_conn = Some(conn);
    host.exec = Some(Box::new(exec));
    if host.status.enabled {
        start_host(&mut host)?;
    }

    Ok(())
}


/// Retrieves current state of the transcoder
pub fn get_state() -> TranscoderState {

    trace!("retrieving transcoder status");
    lock_unwrap!(HOST)
        .status
}


/// Flushes transcoder state to persistent storage
fn flush_settings(host: &Host) -> Result<()> {

    trace!("flushing transcoder settings");
    let conn = host.db_conn.as_ref()
        .ok_or(TranscoderError::Uninitialized)?;
    settings::set(TC_STATUS_SETTING, &host.status, &conn)?;

    Ok(())
}


/// Starts the transcoding process if it is not already running
pub fn enable() -> Result<()> {

    let mut host = lock_unwrap!(HOST);

    if host.status.enabled {
        trace!("transcoder is already running");
        return Ok(());
    }

    trace!("starting transcoder");
    host.status.enabled = true;
    start_host(&mut host)?;
    info!("Enabled transcoder");

    flush_settings(&host)?;

    Ok(())
}


/// Stops the transcoding process if it is running
pub fn disable() -> Result<()> {

    let mut host = lock_unwrap!(HOST);

    if !host.status.enabled {
        trace!("transcoder is not running");
        return Ok(());
    }

    trace!("stopping transcoder");
    host.status.enabled = false;
    stop_host(&mut host)?;
    info!("Disabled transcoder");

    flush_settings(&host)?;

    Ok(())
}


/// Sets orientation of the transcoded stream
///
/// If transcoder is currently enabled, it is restarted using new orientation settings. If the
/// stream already uses the specified orientation, no action is performed.
pub fn set_orientation(orientation: Orientation) -> Result<()> {

    let mut host = lock_unwrap!(HOST);

    if host.status.orientation == orientation {
        trace!("orientation is already {:?}", orientation);
        return Ok(());
    }

    trace!("changing orientation");
    host.status.orientation = orientation;
    if host.status.enabled {
        trace!("restarting transcoder");
        stop_host(&mut host)?;
        start_host(&mut host)?;
        info!("Restarted transcoder after orientation change");
    }

    flush_settings(&host)?;

    Ok(())
}

//#endregion
