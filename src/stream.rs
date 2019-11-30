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
use rand::Rng;
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


        let hls_key_dir = env::var("STATE_DIRECTORY")
            .unwrap_or_else(|_| String::from("."));
        let hls_key_info_path = format!("{}/stream.keyinfo", hls_key_dir);

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
            "-hls_key_info_file", &hls_key_info_path,
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

    trace!("identifying proxy config directory");
    let state_dir = match env::var("STATE_DIRECTORY") {
        Ok(dir) => dir,
        #[cfg(debug_assertions)]
        Err(std::env::VarError::NotPresent) => String::from("."),
        Err(err) => return Err(err.into()),
    };

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


/// Describes an update to the state of a video stream
#[derive(Deserialize, Serialize)]
pub struct StreamUpdate {
    pub enabled: Option<bool>,
    pub orientation: Option<Orientation>,
}


/// Represents the current host's video stream
///
/// This type is used to control the video stream for the current host. To
/// retrieve information about the current stream state, construct a
/// `StreamState` from an instance of `Stream`
pub struct Stream {
    pub(crate) orientation: Orientation,
    pub(crate) transcoder: ProcHost,
    pub(crate) key: [u8; 16],
}

impl Stream {

    /// Retrieves a serializable representation of this stream's state
    pub fn state(&self) -> StreamState {

        StreamState {
            enabled: self.transcoder.running(),
            orientation: self.orientation,
            key: self.key,
        }
    }

    /// Updates this stream's settings
    pub fn update(
        &mut self,
        update: &StreamUpdate,
        conn: &PooledConnection,
        templates: &Tera,
    ) -> Result<()> {

        let mut do_stop = false;
        let mut do_reconfig = false;
        let mut do_start = false;

        if let Some(enabled) = update.enabled {
            if self.transcoder.running() != enabled {
                trace!("updating stream enabled state");
                do_stop = !enabled;
                do_start = enabled;
            }
        }

        if let Some(orientation) = update.orientation {
            if self.orientation != orientation {
                trace!("updating stream orientation");
                self.orientation = orientation;
                do_stop = true;
                do_reconfig = true;
                do_start = true;
            }
        }

        if do_stop {
            debug!("stopping transcoder");
            self.transcoder.stop()?;
            clear_proxy_config()?;
            proxy::reload()?;
        }

        if do_reconfig {
            trace!("reconfiguring transcoder host");
            self.transcoder = ProcHost::new(make_command(self.orientation));
        }

        if do_start {
            debug!("starting transcoder");
            self.transcoder.start()?;
            write_proxy_config(templates)?;
            proxy::reload()?;
        }

        if do_stop || do_reconfig || do_start {
            trace!("flushing stream settings");
            settings::set(STREAM_STATE_SETTING, &self.state(), conn)?;
        }

        Ok(())
    }
}


/// Information about the state of a `Stream`
///
/// This type provides a public/serializable/deserializable representation of
/// the state of a `Stream`. You can retrieve an instance using the `From` trait
/// with an instance of `Stream` or by deserializing the reponse from a
/// *GET /api/stream* API request
#[derive(Deserialize, Serialize)]
pub struct StreamState {
    pub enabled: bool,
    pub orientation: Orientation,
    pub key: [u8; 16],
}

impl Default for StreamState {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            orientation: Default::default(),
            key: rand::thread_rng().gen(),
        }
    }
}


/// Retrieves information about the video stream
fn get_stream(
    stream: Data<RwLock<Stream>>,
) -> Result<Json<StreamState>> {

    let stream = do_read!(stream);

    Ok(Json(stream.state()))
}


/// Updates video stream settings
fn patch_stream(
    pool: Data<ConnectionPool>,
    stream: Data<RwLock<Stream>>,
    templates: Data<Tera>,
    body: Json<StreamUpdate>,
) -> Result<Json<StreamState>> {

    let mut stream = do_write!(stream);
    let conn = pool.get()?;

    stream.update(&body, &conn, &templates)?;

    Ok(Json(stream.state()))
}


/// Initializes an instance of `Stream` for the current host
///
/// This function must be called exactly once over the lifetime of the current
/// process.
pub fn initialize(conn: &PooledConnection, templates: &Tera) -> Result<Stream> {

    trace!("loading stream settings");
    let state: StreamState = match settings::get(STREAM_STATE_SETTING, conn)? {
        Some(state) => state,
        None => {
            let state = Default::default();
            settings::set(STREAM_STATE_SETTING, &state, conn)?;
            state
        }
    };

    // Write configuration files used by FFmpeg to encrypt the HLS
    // stream. For more information, see the FFmpeg docs for hls_key_info_file.
    debug!("configuring HLS encryption");
    let hls_key_dir = env::var("STATE_DIRECTORY")
        .unwrap_or_else(|_| String::from("."));
    let hls_key_path = format!("{}/stream.key", hls_key_dir);
    fs::write(&hls_key_path, state.key)?;
    let hls_key_info = format!("stream.key\n{}\n", hls_key_path);
    let hls_key_info_path = format!("{}/stream.keyinfo", hls_key_dir);
    fs::write(hls_key_info_path, hls_key_info)?;

    trace!("initializing stream");
    let mut transcoder = ProcHost::new(make_command(state.orientation));
    if state.enabled {
        debug!("starting transcoder");
        transcoder.start()?;
        write_proxy_config(templates)?;
    } else {
        clear_proxy_config()?;
    }

    proxy::reload()?;

    Ok(Stream {
        orientation: state.orientation,
        transcoder,
        key: state.key,
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
