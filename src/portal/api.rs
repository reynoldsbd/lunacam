//! Web API

use actix_web::http::{StatusCode};
use actix_web::web::{self, Data, Json, Path, ServiceConfig};
use lunacam::Result;
use lunacam::api::{CameraSettings, UserResource};
use crate::camera::CameraManager;
use crate::Resources;


//#region CRUD for Cameras

fn put_camera(
    resources: Data<Resources>,
    raw: Json<CameraSettings>,
) -> Result<Json<CameraSettings>>
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
    let mut camera = resources.create_camera(hostname, key)?;

    camera.update(raw)?;

    Ok(Json(camera.into()))
}

fn get_camera(
    resources: Data<Resources>,
    path: Path<(i32,)>,
) -> Result<Json<CameraSettings>>
{
    let camera = resources.get_camera(path.0)?;

    Ok(Json(camera.into()))
}

fn get_cameras(
    resources: Data<Resources>,
) -> Result<Json<Vec<CameraSettings>>>
{
    let cameras = resources.get_cameras()?
        .into_iter()
        .map(|cam| cam.into())
        .collect();

    Ok(Json(cameras))
}

fn patch_camera(
    path: Path<(i32,)>,
    raw: Json<CameraSettings>,
    resources: Data<Resources>,
) -> Result<Json<CameraSettings>>
{
    let mut raw = raw.into_inner();
    let mut camera = resources.get_camera(path.0)?;

    // Sanity check
    if raw.id.is_some() && raw.id.take() != Some(camera.id()) {
        Err((StatusCode::BAD_REQUEST, "id mismatch"))?;
    }

    camera.update(raw)?;

    Ok(Json(camera.into()))
}

fn delete_camera(
    path: Path<(i32,)>,
    resources: Data<Resources>,
) -> Result<()>
{
    resources.get_camera(path.0)?
        .delete()?;

    Ok(())
}

//#endregion


//#region CRUD for Users

fn put_user(
    resources: Data<Resources>,
    user: Json<UserResource>,
) -> Result<Json<UserResource>>
{
    // TODO: create and return new user

    Ok(user)
}

fn get_user(
    resources: Data<Resources>,
    path: Path<(i32,)>,
) -> Result<Json<UserResource>>
{
    // TODO: retrieve existing user

    Ok(Json(UserResource {
        display_name: Some("Sample User".into()),
        id: Some(1),
        password: None,
        username: Some("sample".into()),
    }))
}

fn get_users(
    resources: Data<Resources>,
) -> Result<Json<Vec<UserResource>>>
{
    // TODO: retrieve all existing users

    Ok(Json(vec![]))
}

fn patch_user(
    resources: Data<Resources>,
    path: Path<(i32,)>,
    user: Json<UserResource>,
) -> Result<Json<UserResource>>
{
    // TODO: retrieve and update existing user

    Ok(user)
}

fn delete_user(
    resources: Data<Resources>,
    path: Path<(i32,)>,
) -> Result<()>
{
    // TODO: retrieve and delete existing user

    Ok(())
}

//#endregion


/// Configures an Actix service to serve the API
pub fn configure(service: &mut ServiceConfig) {

    service.route("/cameras", web::get().to(get_cameras));
    service.route("/cameras", web::put().to(put_camera));
    service.route("/cameras/{id}", web::get().to(get_camera));
    service.route("/cameras/{id}", web::patch().to(patch_camera));
    service.route("/cameras/{id}", web::delete().to(delete_camera));

    service.route("/users", web::get().to(get_users));
    service.route("/users", web::put().to(put_user));
    service.route("/users/{id}", web::get().to(get_user));
    service.route("/users/{id}", web::patch().to(patch_user));
    service.route("/users/{id}", web::delete().to(delete_user));
}
