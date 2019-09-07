//! Web API

use actix_web::http::{StatusCode};
use actix_web::web::{self, Data, Json, Path, ServiceConfig};
use diesel::prelude::*;
use diesel::sqlite::{SqliteConnection};
use lc_api::{ApiResult, CameraSettings};
use log::{error};
use crate::camera::{Camera};


//#region CRUD for Cameras

fn put_camera(
    db: Data<SqliteConnection>,
    raw: Json<CameraSettings>,
) -> ApiResult<Json<CameraSettings>>
{
    let mut raw = raw.into_inner();

    // Validate input
    if raw.id.is_some() {
        Err((StatusCode::BAD_REQUEST, "cannot specify id when creating new camera resource"))?;
    }

    let hostname = raw.hostname.take()
        .ok_or((StatusCode::BAD_REQUEST, "hostname is required"))?;
    let key = raw.device_key.take()
        .ok_or((StatusCode::BAD_REQUEST, "deviceKey is required"))?;
    let mut camera = Camera::create(hostname, key, &db)?;

    camera.update(raw, &db)?;

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
    let mut raw = raw.into_inner();
    let mut camera = Camera::get(path.0, &db)?;

    // Sanity check
    if raw.id.is_some() && raw.id.take() != Some(camera.id()) {
        Err((StatusCode::BAD_REQUEST, "id mismatch"))?;
    }

    camera.update(raw, &db)?;

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
