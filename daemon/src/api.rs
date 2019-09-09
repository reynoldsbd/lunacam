//! Daemon API

use actix_web::web::{self, Json, ServiceConfig};
use lc_api::StreamSettings;
use lcutil::Result;
use crate::transcoder;


fn get_stream() -> Result<Json<StreamSettings>> {

    Ok(Json(transcoder::get_state().into()))
}

fn patch_stream(stream: Json<StreamSettings>) -> Result<Json<StreamSettings>> {

    if let Some(enabled) = stream.enabled {
        if enabled {
            transcoder::enable()?;
        } else {
            transcoder::disable()?;
        }
    }

    if let Some(orientation) = stream.orientation {
        transcoder::set_orientation(orientation)?;
    }

    Ok(Json(transcoder::get_state().into()))
}


pub fn configure(service: &mut ServiceConfig) {

    service.route("/stream", web::get().to(get_stream));
    service.route("/stream", web::patch().to(patch_stream));
}
