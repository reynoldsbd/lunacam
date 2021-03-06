//! Camera management


use std::fs;
use std::path::Path;
use std::sync::RwLock;

use actix_web::http::{StatusCode};
use actix_web::web::{self, Data, Json, ServiceConfig};
use diesel::prelude::*;
use log::{debug, error, info, trace, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::do_write;
use crate::db::{ConnectionPool, PooledConnection};
use crate::db::schema::cameras;
use crate::error::{Error, Result};
use crate::proxy;
use crate::stream::{Orientation, Stream, StreamState, StreamUpdate};
use crate::users::AuthenticationMiddleware;


/// Representation of a streaming camera
#[derive(Serialize)]
#[derive(AsChangeset, Identifiable, Queryable)]
#[table_name = "cameras"]
pub struct Camera {
    pub id: i32,
    pub name: String,
    pub address: String,
    pub enabled: bool,
    pub orientation: Orientation,
    pub local: bool,
    pub key: Vec<u8>,
}


/// Camera representation required by PUT requests
#[derive(Deserialize)]
struct PutCameraBody {
    name: String,
    address: String,
}


#[derive(Insertable)]
#[table_name = "cameras"]
struct NewCamera<'a> {
    name: &'a str,
    address: &'a str,
    enabled: bool,
    orientation: Orientation,
    local: bool,
    key: &'a [u8],
}


/// Creates a new camera
fn put_camera(
    client: Data<Client>,
    pool: Data<ConnectionPool>,
    templates: Data<Tera>,
    body: Json<PutCameraBody>,
) -> Result<Json<Camera>>
{
    // Validate connection before touching the database
    debug!("connecting to camera at {}", body.address);
    let url = format!("http://{}/api/stream", body.address);
    let stream: StreamState = client.get(&url)
        .send()?
        .json()?;

    debug!("adding new camera to database");
    let conn = pool.get()?;
    let new_cam = NewCamera {
        name: &body.name,
        address: &body.address,
        enabled: stream.enabled,
        orientation: stream.orientation,
        local: false,
        key: &stream.key,
    };
    diesel::insert_into(cameras::table)
        .values(&new_cam)
        .execute(&conn)?;

    // Get the row we just inserted
    let camera: Camera = cameras::table.order(cameras::id.desc())
        .first(&conn)?;

    if camera.enabled {
        write_proxy_config(&camera, &client, &templates)
            .and_then(|_| proxy::reload())
            .unwrap_or_else(|e|
                error!("failed to configure proxy for camera {}: {}", camera.id, e)
            );
    }

    info!("created new camera {}", camera.id);

    Ok(Json(camera))
}


/// Retrieves information about the specified camera
fn get_camera(
    pool: Data<ConnectionPool>,
    path: web::Path<(i32,)>,
) -> Result<Json<Camera>>
{
    let id = path.0;

    debug!("retrieving camera {} from database", id);
    let conn = pool.get()?;
    let camera: Camera = cameras::table.find(id)
        .get_result(&conn)?;

    Ok(Json(camera))
}


/// Retrieves information about all cameras
fn get_cameras(
    pool: Data<ConnectionPool>,
) -> Result<Json<Vec<Camera>>>
{
    debug!("retrieving all cameras from database");
    let conn = pool.get()?;
    let cameras = cameras::table.load(&conn)?;

    Ok(Json(cameras))
}


/// Camera representation required by PATCH requests
#[derive(Deserialize)]
struct PatchCameraBody {
    name: Option<String>,
    enabled: Option<bool>,
    orientation: Option<Orientation>,
    address: Option<String>,
}


/// Updates information about the specified camera
#[allow(clippy::assertions_on_constants)]
#[allow(clippy::cognitive_complexity)]
fn patch_camera(
    pool: Data<ConnectionPool>,
    client: Data<Client>,
    templates: Data<Tera>,
    #[cfg(feature = "stream")]
    stream: Data<RwLock<Stream>>,
    path: web::Path<(i32,)>,
    body: Json<PatchCameraBody>,
) -> Result<Json<Camera>>
{
    let id = path.0;
    let body = body.into_inner();

    debug!("retrieving camera {} from database", id);
    let conn = pool.get()?;
    let mut camera: Camera = cameras::table.find(id)
        .get_result(&conn)?;

    let mut do_connect = false;
    let mut do_update = false;
    let mut do_save = false;
    let mut new_stream = StreamUpdate {
        enabled: None,
        orientation: None,
    };

    if let Some(name) = body.name {
        if camera.name != name {
            trace!("updating name for camera {}", id);
            camera.name = name;
            do_save = true;
        }
    }

    if let Some(enabled) = body.enabled {
        if camera.enabled != enabled {
            trace!("updating enabled for camera {}", id);
            camera.enabled = enabled;
            new_stream.enabled = Some(enabled);
            do_update = true;
            do_save = true;
        }
    }

    if let Some(orientation) = body.orientation {
        if camera.orientation != orientation {
            trace!("updating orientation for camera {}", id);
            camera.orientation = orientation;
            new_stream.orientation = Some(orientation);
            do_update = true;
            do_save = true;
        }
    }

    if let Some(address) = body.address {
        if camera.local {
            return Error::web(
                StatusCode::BAD_REQUEST,
                "cannot update address of local camera",
            );
        }
        if camera.address != address {
            trace!("updating address for camera {}", id);
            camera.address = address;
            do_connect = true;
            do_save = true;
        }
    }

    // Validate new connection information before updating the database
    let url = format!("http://{}/api/stream", camera.address);
    if do_connect {
        debug!("connecting to camera at {}", camera.address);
        let current_stream: StreamState = client.get(&url)
            .send()?
            .json()?;

        // If successful, update the camera instance to reflect the settings of the connected
        // device. As an optimization, we skip these updates if we're about to change one of these
        // settings.
        if new_stream.enabled.is_none() {
            trace!("updating enabled for camera {}", id);
            camera.enabled = current_stream.enabled;
            do_save = true;
        }
        if new_stream.orientation.is_none() {
            trace!("updating orientation for camera {}", id);
            camera.orientation = current_stream.orientation;
            do_save = true;
        }
    }

    if do_update {
        if camera.local {
            assert!(cfg!(feature = "stream"));
            debug!("updating local stream settings");
            #[cfg(feature = "stream")]
            do_write!(stream).update(&new_stream, &conn, &templates)?;
        } else {
            debug!("sending new stream settings to {}", camera.address);
            client.patch(&url)
                .json(&new_stream)
                .send()?;
        }
    }

    if do_save {
        debug!("saving changes to camera {}", id);
        diesel::update(&camera)
            .set(&camera)
            .execute(&conn)?;
        if camera.enabled {
            write_proxy_config(&camera, &client, &templates)
                .unwrap_or_else(|e|
                    error!("failed to configure proxy for camera {}: {}", camera.id, e)
                );
        } else {
            clear_proxy_config(camera.id)
                .unwrap_or_else(|e|
                    error!("failed to clear proxy configuration for camera {}: {}", camera.id, e)
                );
        }
        proxy::reload()
            .unwrap_or_else(|e| error!("failed to reload proxy configuration: {}", e));
    }

    info!("successfully updated camera {}", id);
    Ok(Json(camera))
}


/// Deletes the specified camera
fn delete_camera(
    pool: Data<ConnectionPool>,
    path: web::Path<(i32,)>,
) -> Result<()>
{
    let id = path.0;
    let conn = pool.get()?;

    debug!("retrieving camera {} from database", id);
    let camera: Camera = cameras::table.find(id)
        .get_result(&conn)?;
    if camera.local {
        return Error::web(
            StatusCode::BAD_REQUEST,
            "cannot delete local camera",
        );
    }

    debug!("deleting camera {} from database", id);
    diesel::delete(cameras::table.filter(cameras::id.eq(id)))
        .execute(&conn)?;

    clear_proxy_config(id)
        .and_then(|_| proxy::reload())
        .unwrap_or_else(|e| error!("failed to clear proxy configuration for camera {}: {}", id, e));

    info!("deleted camera {}", id);

    Ok(())
}


/// Configures the */cameras* API resource
pub fn configure_api(service: &mut ServiceConfig) {

    service.service(
        web::resource("/cameras")
            .route(web::get().to(get_cameras))
            .route(web::put().to(put_camera))
            .wrap(AuthenticationMiddleware::reject())
    );

    service.service(
        web::resource("/cameras/{id}")
            .route(web::get().to(get_camera))
            .route(web::patch().to(patch_camera))
            .route(web::delete().to(delete_camera))
            .wrap(AuthenticationMiddleware::reject())
    );
}


/// Gets path of the proxy configuration file for the specified camera
fn get_proxy_config_path(id: i32) -> Result<impl AsRef<Path>> {

    let cfg_dir = proxy::config_dir()?;
    let path = format!("{}/proxy-{}.conf", cfg_dir, id);

    Ok(path)
}


/// Writes or removes the proxy configuration file for this camera
fn write_proxy_config(
    camera: &Camera,
    client: &Client,
    templates: &Tera
) -> Result<()> {

    // Validate remote address before writing proxy config (otherwise, Nginx
    // will fail to start/reload, potentially making the web UI inaccessible)
    if !camera.local {
        debug!("validating connection to camera {}", camera.id);
        let url = format!("http://{}/api/stream", camera.address);
        if let Err(err) = client.get(&url).send() {
            // Misconfigured camera should not bring down the whole system
            error!("failed to connect to camera {}: {}", camera.id, err);
            warn!("skipping proxy configuration");
            return Ok(());
        }
    }

    let mut context = Context::new();
    context.insert("camera", camera);
    let config = templates.render("proxy.conf", &context)?;

    debug!("writing proxy configuration for camera {}", camera.id);
    let config_path = get_proxy_config_path(camera.id)?;
    fs::write(&config_path, config)?;

    Ok(())
}


/// Removes proxy configuration for the specified camera
fn clear_proxy_config(id: i32) -> Result<()> {

    let config_path = get_proxy_config_path(id)?;

    if fs::metadata(&config_path).is_ok() {
        debug!("clearing proxy configuration for camera {}", id);
        fs::remove_file(&config_path)?;
    }

    Ok(())
}


/// Initializes the `cameras` module
///
/// Performs the following operations to make this module usable:
///
/// * Ensures proxy is properly configured
/// * Ensures locally-attached cameras are properly identified and registered in
///   the database
///
/// This function must be called exactly once before using the rest of the APIs
/// in this module.
pub fn initialize(
    conn: &PooledConnection,
    client: &Client,
    templates: &Tera,
    #[cfg(feature = "stream")]
    stream: &Stream,
) -> Result<()> {

    let cameras: Vec<Camera> = cameras::table.load(conn)?;

    for camera in &cameras {
        if camera.enabled {
            write_proxy_config(&camera, client, templates)
                .unwrap_or_else(|e|
                    error!("failed to configure proxy for camera {}: {}", camera.id, e)
                );
        } else {
            clear_proxy_config(camera.id)
                .unwrap_or_else(|e|
                    error!("failed to clear proxy configuration for camera {}: {}", camera.id, e)
                );
        }
    }

    proxy::reload()
        .unwrap_or_else(|e| error!("failed to reload proxy configuration: {}", e));

    #[cfg(feature = "stream")]
    {
        let local_cam_count = cameras.iter()
            .filter(|c| c.local)
            .count();

        assert!(local_cam_count <= 1);

        if local_cam_count == 0 {
            info!("initializing local camera");
            let local_cam = NewCamera {
                name: "Local Camera",
                address: "",
                enabled: stream.transcoder.running(),
                orientation: stream.orientation,
                local: true,
                key: &stream.key,
            };
            diesel::insert_into(cameras::table)
                .values(&local_cam)
                .execute(conn)?;
        }
    }

    Ok(())
}


/// Retrieves serializable representation of all cameras
pub fn all(conn: &PooledConnection) -> Result<impl Serialize> {

    let users: Vec<Camera> = cameras::table.load(conn)?;

    Ok(users)
}


/// Retrieves serializable representation of the specified camera
pub fn get(id: i32, conn: &PooledConnection) -> Result<Camera> {

    let user = cameras::table.find(id)
        .get_result(conn)?;

    Ok(user)
}
