//! Video stream management

// Actix handlers have lots of needless pass-by-value (Data, Json, and Path structs)
#![allow(clippy::needless_pass_by_value)]

use std::process::{Command, Stdio};
use std::sync::RwLock;

use actix_web::web::{self, Data, Json, ServiceConfig};
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{do_read, do_write};
use crate::error::Result;
use crate::api::{Orientation};
use crate::db::{ConnectionPool, PooledConnection};
use crate::prochost::ProcHost;
use crate::settings;


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
const STREAM_STATE_SETTING: &str = "streamState";


pub struct Stream {
    orientation: Orientation,
    transcoder: ProcHost,
}


#[derive(Default, Deserialize, Serialize)]
struct StreamState {
    enabled: bool,
    orientation: Orientation,
}

impl From<&Stream> for StreamState {
    fn from(stream: &Stream) -> Self {
        Self {
            enabled: stream.transcoder.running(),
            orientation: stream.orientation,
        }
    }
}


/// Retrieves information about the video stream
fn get_stream(
    stream: Data<RwLock<Stream>>,
) -> Result<Json<StreamState>> {

    let stream = do_read!(stream);
    let settings = (&*stream).into();

    Ok(Json(settings))
}


#[derive(Deserialize)]
struct PatchStreamBody {
    enabled: Option<bool>,
    orientation: Option<Orientation>,
}


/// Updates video stream settings
fn patch_stream(
    pool: Data<ConnectionPool>,
    stream: Data<RwLock<Stream>>,
    body: Json<PatchStreamBody>,
) -> Result<Json<StreamState>> {

    let mut stream = do_write!(stream);

    let mut do_stop = false;
    let mut do_reconfig = false;
    let mut do_start = false;

    if let Some(enabled) = body.enabled {
        if stream.transcoder.running() != enabled {
            trace!("updating stream enabled state");
            do_stop = !enabled;
            do_start = enabled;
        }
    }

    if let Some(orientation) = body.orientation {
        if stream.orientation != orientation {
            trace!("updating stream orientation");
            stream.orientation = orientation;
            do_stop = true;
            do_reconfig = true;
            do_start = true;
        }
    }

    if do_stop {
        debug!("stopping transcoder");
        stream.transcoder.stop()?;
    }

    if do_reconfig {
        trace!("reconfiguring transcoder host");
        stream.transcoder = ProcHost::new(make_command(stream.orientation));
    }

    if do_start {
        debug!("starting transcoder");
        stream.transcoder.start()?;
    }

    let settings = (&*stream).into();

    if do_stop || do_reconfig || do_start {
        trace!("flushing stream settings");
        let conn = pool.get()?;
        settings::set(STREAM_STATE_SETTING, &settings, &conn)?;
    }

    Ok(Json(settings))
}


pub fn initialize(conn: &PooledConnection) -> Result<Stream> {

    let settings: StreamState = settings::get(STREAM_STATE_SETTING, conn)?
        .unwrap_or_default();

    let mut transcoder = ProcHost::new(make_command(settings.orientation));
    if settings.enabled {
        debug!("starting transcoder");
        transcoder.start()?;
    }

    Ok(Stream {
        orientation: settings.orientation,
        transcoder,
    })
}


/// Configures the */stream* API resource
pub fn configure_api(service: &mut ServiceConfig) {

    service.service(
        web::resource("/stream")
            .route(web::get().to(get_stream))
            .route(web::patch().to(patch_stream))
    );
}
