//! Web API

use actix_web::http::{StatusCode};
use actix_web::web::{self, Data, Json, Path, ServiceConfig};
use lc_api::{ApiResult, CameraSettings};
use crate::{ConnectionPool};
use crate::camera::{Camera};


//#region CRUD for Cameras

fn put_camera(
    pool: Data<ConnectionPool>,
    raw: Json<CameraSettings>,
) -> ApiResult<Json<CameraSettings>>
{
    let conn = pool.get()?;
    let mut raw = raw.into_inner();

    // Validate input
    if raw.id.is_some() {
        Err((StatusCode::BAD_REQUEST, "cannot specify id when creating new camera resource"))?;
    }

    let hostname = raw.hostname.take()
        .ok_or((StatusCode::BAD_REQUEST, "hostname is required"))?;
    let key = raw.device_key.take()
        .ok_or((StatusCode::BAD_REQUEST, "deviceKey is required"))?;
    let mut camera = Camera::create(hostname, key, &conn)?;

    camera.update(raw, &conn)?;

    Ok(Json(camera.into()))
}

fn get_camera(
    path: Path<(i32,)>,
    pool: Data<ConnectionPool>,
) -> ApiResult<Json<CameraSettings>>
{
    let conn = pool.get()?;
    Ok(Json(Camera::get(path.0, &conn)?.into()))
}

fn get_cameras(
    pool: Data<ConnectionPool>,
) -> ApiResult<Json<Vec<CameraSettings>>>
{
    let conn = pool.get()?;
    let cameras = Camera::get_all(&conn)?
        .into_iter()
        .map(|cam| cam.into())
        .collect();

    Ok(Json(cameras))
}

fn patch_camera(
    path: Path<(i32,)>,
    raw: Json<CameraSettings>,
    pool: Data<ConnectionPool>,
) -> ApiResult<Json<CameraSettings>>
{
    let conn = pool.get()?;
    let mut raw = raw.into_inner();
    let mut camera = Camera::get(path.0, &conn)?;

    // Sanity check
    if raw.id.is_some() && raw.id.take() != Some(camera.id()) {
        Err((StatusCode::BAD_REQUEST, "id mismatch"))?;
    }

    camera.update(raw, &conn)?;

    Ok(Json(camera.into()))
}

fn delete_camera(
    path: Path<(i32,)>,
    pool: Data<ConnectionPool>,
) -> ApiResult<()>
{
    let conn = pool.get()?;
    Camera::get(path.0, &conn)?
        .delete(&conn)?;

    Ok(())
}

//#endregion


/// Configures an Actix service to serve the API
pub fn configure(pool: ConnectionPool) -> impl FnOnce(&mut ServiceConfig)
{
    move |service| {
        service.data(pool);

        service.route("/cameras", web::get().to(get_cameras));
        service.route("/cameras", web::put().to(put_camera));
        service.route("/cameras/{id}", web::get().to(get_camera));
        service.route("/cameras/{id}", web::patch().to(patch_camera));
        service.route("/cameras/{id}", web::delete().to(delete_camera));
    }
}
