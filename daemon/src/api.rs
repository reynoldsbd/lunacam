//! Daemon API

use actix_web::web::{self, Json, ServiceConfig};
use serde::{Deserialize, Serialize};
use crate::Orientation;
use crate::transcoder::{self, TranscoderState};


//#region Stream Resource

#[derive(Deserialize, Serialize)]
struct StreamResource {
    enabled: Option<bool>,
    orientation: Option<Orientation>,
}

impl From<TranscoderState> for StreamResource {
    fn from(status: TranscoderState) -> Self {
        Self {
            enabled: Some(status.enabled),
            orientation: Some(status.orientation),
        }
    }
}


fn get_stream() -> Json<StreamResource> {

    Json(transcoder::get_state().into())
}

fn patch_stream(raw: Json<StreamResource>) -> Json<StreamResource> {

    if let Some(enabled) = raw.enabled {
        if enabled {
            transcoder::enable().unwrap();
        } else {
            transcoder::disable().unwrap();
        }
    }

    if let Some(orientation) = raw.orientation {
        transcoder::set_orientation(orientation).unwrap();
    }

    Json(transcoder::get_state().into())
}

//#endregion


pub fn configure(service: &mut ServiceConfig) {

    service.route("/stream", web::get().to(get_stream));
    service.route("/stream", web::patch().to(patch_stream));
}
