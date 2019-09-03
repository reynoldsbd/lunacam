//! Web API

use actix_web::http::{StatusCode};
use actix_web::web::{self, Data, Json, Path, ServiceConfig};
use diesel::prelude::*;
use diesel::sqlite::{SqliteConnection};
use lc_api::{ApiResult, CameraSettings};
use log::{error};
use crate::camera::{Camera, NewCamera};


//#region CRUD for Cameras

fn put_camera(
    db: Data<SqliteConnection>,
    raw: Json<CameraSettings>,
) -> ApiResult<Json<CameraSettings>>
{
    let raw = raw.into_inner();

    // Validate input
    if raw.id.is_some() {
        Err((StatusCode::BAD_REQUEST, "cannot specify id when creating new camera resource"))?;
    }
    let new_camera = NewCamera {
        hostname: raw.hostname
            .ok_or((StatusCode::BAD_REQUEST, "hostname is required"))?,
        device_key: raw.device_key
            .ok_or((StatusCode::BAD_REQUEST, "device_key is required"))?,
        friendly_name: raw.friendly_name
            .ok_or((StatusCode::BAD_REQUEST, "friendly_name is required"))?,
    };

    let mut camera = Camera::create(new_camera, &db)?;

    // TODO: replace with encapsulated camera api
    let mut save_needed = false;
    if let Some(enabled) = raw.enabled {
        camera.enabled = enabled;
        save_needed = true;
    }
    if let Some(orientation) = raw.orientation {
        camera.orientation = orientation;
        save_needed = true;
    }
    if save_needed {
        camera.save(&db)?;
    }

    Ok(Json(camera.into()))
}

fn get_camera(
    path: Path<(i32,)>,
    db: Data<SqliteConnection>,
) -> ApiResult<Json<CameraSettings>>
{
    Ok(Json(Camera::get(path.0, &db)?.into()))
}

fn get_cameras(
    db: Data<SqliteConnection>,
) -> ApiResult<Json<Vec<CameraSettings>>>
{
    let cameras = Camera::get_all(&db)?
        .into_iter()
        .map(|cam| cam.into())
        .collect();

    Ok(Json(cameras))
}

fn patch_camera(
    path: Path<(i32,)>,
    raw: Json<CameraSettings>,
    db: Data<SqliteConnection>,
) -> ApiResult<Json<CameraSettings>>
{
    let raw = raw.into_inner();
    let mut camera = Camera::get(path.0, &db)?;

    // Sanity check
    if raw.id.is_some() && raw.id != Some(camera.id) {
        Err((StatusCode::NOT_FOUND, "id mismatch"))?;
    }

    if let Some(enabled) = raw.enabled {
        camera.enabled = enabled;
    }
    if let Some(hostname) = raw.hostname {
        camera.hostname = hostname;
    }
    if let Some(device_key) = raw.device_key {
        camera.device_key = device_key;
    }
    if let Some(friendly_name) = raw.friendly_name {
        camera.friendly_name = friendly_name;
    }
    if let Some(orientation) = raw.orientation {
        camera.orientation = orientation;
    }
    camera.save(&db)?;

    Ok(Json(camera.into()))
}

fn delete_camera(
    path: Path<(i32,)>,
    db: Data<SqliteConnection>,
) -> ApiResult<()>
{
    Camera::get(path.0, &db)?
        .delete(&db)?;

    Ok(())
}

//#endregion


/// Configures an Actix service to serve the API
pub fn configure(db_url: String) -> impl Fn(&mut ServiceConfig)
{
    move |service| {
        match SqliteConnection::establish(&db_url) {
            Ok(conn) => { service.data(conn); },
            Err(err) => error!("Failed to connect to database: {}", err),
        }

        service.route("/cameras", web::get().to(get_cameras));
        service.route("/cameras", web::put().to(put_camera));
        service.route("/cameras/{id}", web::get().to(get_camera));
        service.route("/cameras/{id}", web::patch().to(patch_camera));
        service.route("/cameras/{id}", web::delete().to(delete_camera));
    }
}
