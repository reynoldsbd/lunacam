//! Daemon API

use actix_web::web::{self, Json, ServiceConfig};
use lc_api::{ApiResult, StreamSettings};
use crate::transcoder;


fn get_stream() -> ApiResult<Json<StreamSettings>> {

    Ok(Json(transcoder::get_state().into()))
}

fn patch_stream(stream: Json<StreamSettings>) -> ApiResult<Json<StreamSettings>> {

    if let Some(enabled) = stream.enabled {
        if enabled {
            transcoder::enable().unwrap();
        } else {
            transcoder::disable().unwrap();
        }
    }

    if let Some(orientation) = stream.orientation {
        transcoder::set_orientation(orientation).unwrap();
    }

    Ok(Json(transcoder::get_state().into()))
}


pub fn configure(service: &mut ServiceConfig) {

    service.route("/stream", web::get().to(get_stream));
    service.route("/stream", web::patch().to(patch_stream));
}
