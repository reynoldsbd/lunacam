//! Video stream management
//!
//! Each host in a LunaCam network may or may not expose a video stream. This
//! module controls the properties and lifecycle of the current host's video
//! stream.


use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::RwLock;

use actix_web::web::{self, Data, Json, ServiceConfig};
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Integer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::{do_read, do_write};
use crate::error::Result;
use crate::db::{ConnectionPool, PooledConnection};
use crate::prochost::ProcHost;
use crate::proxy;
use crate::settings;


//#region Orientation

/// Video stream orientation
#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(AsExpression, FromSqlRow)]
#[sql_type = "Integer"]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Orientation {
    Landscape,
    Portrait,
    InvertedLandscape,
    InvertedPortrait,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Landscape
    }
}

impl<B> FromSql<Integer, B> for Orientation
where
    B: Backend,
    i32: FromSql<Integer, B>,
{
    fn from_sql(bytes: Option<&B::RawValue>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            0 => Ok(Self::Landscape),
            1 => Ok(Self::Portrait),
            2 => Ok(Self::InvertedLandscape),
            3 => Ok(Self::InvertedPortrait),
            other => Err(format!("Unrecognized value \"{}\"", other).into()),
        }
    }
}

impl<B> ToSql<Integer, B> for Orientation
where
    B: Backend,
    i32: ToSql<Integer, B>,
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, B>) -> serialize::Result {
        let val = match *self {
            Self::Landscape => 0,
            Self::Portrait => 1,
            Self::InvertedLandscape => 2,
            Self::InvertedPortrait => 3,
        };

        val.to_sql(out)
    }
}

//#endregion


/// Creates a `Command` for starting the transcoder
fn make_command(_orientation: Orientation) -> Command {

    // In debug mode, start a dummy process
    let mut cmd = if cfg!(debug_assertions) {

        let mut cmd = Command::new("sh");
        let state_dir = env::var("STATE_DIRECTORY")
            .unwrap_or_else(|_| String::from("."));
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


/// Gets the location of the HLS stream's proxy configuration file
fn get_proxy_config_path() -> Result<impl AsRef<Path>> {

    let state_dir = env::var("STATE_DIRECTORY")?;
    let path = format!("{}/nginx/hls.conf", state_dir);

    Ok(path)
}


/// Writes proxy configuration for the HLS stream
fn write_proxy_config(templates: &Tera) -> Result<()> {

    debug!("writing proxy configuration for HLS stream");

    let config = templates.render("hls.conf", Context::new())?;

    let config_path = get_proxy_config_path()?;

    fs::write(&config_path, config)?;

    Ok(())
}


/// Removes proxy configuration for the HLS stream
fn clear_proxy_config() -> Result<()> {

    let config_path = get_proxy_config_path()?;

    if fs::metadata(&config_path).is_ok() {
        debug!("clearing proxy configuration for HLS stream");
        fs::remove_file(&config_path)?;
    }

    Ok(())
}


/// Key used to store stream settings
const STREAM_STATE_SETTING: &str = "streamState";


/// Represents the current host's video stream
///
/// This type is used to control the video stream for the current host. To
/// retrieve information about the current stream state, construct a
/// `StreamState` from an instance of `Stream`
pub struct Stream {
    orientation: Orientation,
    transcoder: ProcHost,
}


/// Information about the state of a `Stream`
///
/// This type provides a public/serializable/deserializable representation of
/// the state of a `Stream`. You can retrieve an instance using the `From` trait
/// with an instance of `Stream` or by deserializing the reponse from a
/// *GET /api/stream* API request
#[derive(Default, Deserialize, Serialize)]
pub struct StreamState {
    pub enabled: bool,
    pub orientation: Orientation,
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


/// Stream representation required by PATCH requests
#[derive(Deserialize, Serialize)]
pub struct PatchStreamBody {
    pub enabled: Option<bool>,
    pub orientation: Option<Orientation>,
}


/// Updates video stream settings
fn patch_stream(
    pool: Data<ConnectionPool>,
    stream: Data<RwLock<Stream>>,
    templates: Data<Tera>,
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
        clear_proxy_config()?;
        proxy::reload()?;
    }

    if do_reconfig {
        trace!("reconfiguring transcoder host");
        stream.transcoder = ProcHost::new(make_command(stream.orientation));
    }

    if do_start {
        debug!("starting transcoder");
        stream.transcoder.start()?;
        write_proxy_config(&templates)?;
        proxy::reload()?;
    }

    let settings = (&*stream).into();

    if do_stop || do_reconfig || do_start {
        trace!("flushing stream settings");
        let conn = pool.get()?;
        settings::set(STREAM_STATE_SETTING, &settings, &conn)?;
    }

    Ok(Json(settings))
}


/// Initializes an instance of `Stream` for the current host
///
/// This function must be called exactly once over the lifetime of the current
/// process.
pub fn initialize(conn: &PooledConnection, templates: &Tera) -> Result<Stream> {

    let settings: StreamState = settings::get(STREAM_STATE_SETTING, conn)?
        .unwrap_or_default();

    let mut transcoder = ProcHost::new(make_command(settings.orientation));
    if settings.enabled {
        debug!("starting transcoder");
        transcoder.start()?;
        write_proxy_config(templates)?;
    } else {
        clear_proxy_config()?;
    }

    proxy::reload()?;

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
