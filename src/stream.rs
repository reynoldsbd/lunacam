//! Video stream management

use std::process::{Command, Stdio};
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::api::{Orientation, StreamSettings};
use crate::db::ConnectionPool;
use crate::prochost::ProcHost;
use crate::settings;


/// Current state of the stream
#[derive(Default)]
#[derive(Deserialize, Serialize)]
struct State {
    enabled: bool,
    orientation: Orientation,
}


/// Creates a `Command` for starting the transcoder
fn make_command(_orientation: Orientation) -> Command {

    // In debug mode, start a dummy process
    let mut cmd = if cfg!(debug_assertions) {

        let mut cmd = Command::new("sh");
        let state_dir = std::env::var("STATE_DIRECTORY").unwrap();
        cmd.arg("-c");
        cmd.arg(format!("while : ; do date > {}/time.txt; sleep 1; done", state_dir));
        cmd

    // In release mode, start the actual transcoder process
    } else {

        // TODO: parameterize orientation, output path, ...
        let mut cmd = Command::new("ffmpeg");
        cmd.args(&[
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
            "/tmp/lunacam/hls/stream.m3u8",
        ]);
        cmd
    };

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    cmd
}


/// Key used to store stream settings
const STATE_SETTING: &str = "streamState";


/// Represents a video stream
///
/// An external program (FFmpeg) is used to capture raw camera frames and transcode them into a
/// suitable streaming format.
#[allow(clippy::module_name_repetitions)] // "Stream" is too generic
pub struct VideoStream {
    state: State,
    pool: ConnectionPool,
    host: ProcHost,
}

impl VideoStream {

    /// Creates the `VideoStream`
    ///
    /// Loads settings from persistent storage and initializes the stream according to that state.
    ///
    /// All instances of `VideoStream` currently use the same hard-coded settings key. Because of
    /// this, only one instance of `VideoStream` should ever exist at a given time.
    ///
    /// In order to support multiple video streams per LunaCam node, additional work will be needed
    /// to ensure each `VideoStream` has unique state.
    pub fn new(pool: ConnectionPool) -> Result<Self> {

        let conn = pool.get()?;
        let state: State = settings::get(STATE_SETTING, &conn)?
            .unwrap_or_default();

        let mut host = ProcHost::new(make_command(state.orientation));
        if state.enabled {
            info!("starting stream");
            host.start()?;
        }

        Ok(Self {
            state,
            pool,
            host
        })
    }

    /// Updates stream settings
    pub fn update(&mut self, settings: StreamSettings) -> Result<()> {

        let mut do_stop = false;
        let mut do_reconfig = false;
        let mut do_start = true;

        if let Some(enabled) = settings.enabled {
            if self.state.enabled != enabled {
                self.state.enabled = enabled;
                do_stop = !enabled;
                do_start = enabled;
            }
        }

        if let Some(orientation) = settings.orientation {
            if self.state.orientation != orientation {
                self.state.orientation = orientation;
                do_stop = true;
                do_reconfig = true;
                do_start = true;
            }
        }

        if do_stop {
            info!("stopping stream");
            self.host.stop()?;
        }

        if do_reconfig {
            debug!("reconfiguring transcoder host");
            self.host = ProcHost::new(make_command(self.state.orientation));
        }

        if do_start {
            info!("starting stream");
            self.host.start()?;
        }

        if do_stop || do_reconfig || do_start {
            trace!("flushing stream settings");
            let conn = self.pool.get()?;
            settings::set(STATE_SETTING, &self.state, &conn)?;
        }

        Ok(())
    }

    /// Retrieves current state of the stream
    pub fn settings(&self) -> StreamSettings {

        StreamSettings {
            enabled: Some(self.state.enabled),
            orientation: Some(self.state.orientation),
        }
    }
}
