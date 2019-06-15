//! Video stream management


//#region Usings

use serde::{Deserialize, Serialize};

use crate::config::Config;

//#endregion


//#region Configuration

/// Operational parameters for the video stream
#[derive(Default, Deserialize, Serialize)]
struct StreamConfiguration
{
    enabled: bool,
}

//#endregion


//#region Stream Manager

/// Manages lifecycle of the video stream
pub struct StreamManager
{
    config: Config<StreamConfiguration>,
}

impl StreamManager
{
    /// Initializes and returns `StreamManager`
    pub fn load() -> StreamManager
    {
        StreamManager {
            config: Config::new("stream")
                .expect("Failed to load stream configuration"),
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

        // TODO: stop/start the transcoding process
    }
}

//#endregion
