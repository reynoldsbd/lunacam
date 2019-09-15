//! Daemon API

use std::sync::Mutex;
use actix_web::web::{self, Data, Json, ServiceConfig};
use lunacam::{do_lock, Result};
use lunacam::api::StreamSettings;
use lunacam::stream::VideoStream;


fn get_stream(stream: Data<Mutex<VideoStream>>) -> Result<Json<StreamSettings>> {

    let stream = do_lock!(stream);

    Ok(Json(stream.settings()))
}

fn patch_stream(
    stream: Data<Mutex<VideoStream>>,
    settings: Json<StreamSettings>,
) -> Result<Json<StreamSettings>> {

    let mut stream = do_lock!(stream);

    stream.update(&settings)?;

    Ok(Json(stream.settings()))
}


pub fn configure(service: &mut ServiceConfig) {

    service.route("/stream", web::get().to(get_stream));
    service.route("/stream", web::patch().to(patch_stream));
}
