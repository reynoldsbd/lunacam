//! Video stream management

// TODO: error handling
// TODO: futures


//#region Usings

use std::process::{Command};

use actix::{Actor, Addr};

use log::{debug};

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::prochost::{ProcessHost, StartProcess, StopProcess};

//#endregion


//#region Configuration

/// Operational parameters for the video stream
#[derive(Default, Deserialize, Serialize)]
pub struct StreamConfiguration
{
    pub enabled: bool,
}

impl StreamConfiguration
{
    fn cmd(&self) -> Command
    {
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
            "/tmp/lunacam/hls/video0.m3u8"
        ]);

        cmd
    }
}

//#endregion


//#region Stream Manager

/// Manages lifecycle of the video stream
pub struct StreamManager
{
    // TODO: would be cool if this didn't have to be pub
    pub config: Config<StreamConfiguration>,

    tchost: Addr<ProcessHost>,
}

impl StreamManager
{
    /// Initializes and returns `StreamManager`
    pub fn load() -> StreamManager
    {
        let config: Config<StreamConfiguration> = Config::new("stream")
            .expect("Failed to load stream configuration");
        let tchost = ProcessHost::new(config.read().cmd())
            .start();

        {
            let config = config.read();
            if config.enabled {
                tchost.do_send(StartProcess);
            }
        }

        StreamManager {
            config: config,
            tchost: tchost,
        }
    }

    /// Sets whether the stream is currently enabled
    ///
    /// When enabled, the underlying transcoding process is running and clients may access the
    /// stream. When disabled, the transcoding process is stopped and the stream is unavailable.
    pub fn set_enabled(&self, enabled: bool)
    {
        {
            let mut config = self.config.write();
            config.enabled = enabled;
        }

        if enabled {
            debug!("starting transcoder");
            self.tchost.do_send(StartProcess);

        } else {
            debug!("stopping transcoder");
            self.tchost.do_send(StopProcess);
        }
    }
}

//#endregion
