use std::sync::Mutex;
use actix_web::{App, HttpServer};
use actix_web::web::{self, Data, Json};
use lunacam::{do_lock, Result};
use lunacam::api::StreamSettings;
use lunacam::db;
use lunacam::logging;
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


fn main() -> Result<()> {

    logging::init();

    let pool = db::connect()?;
    let stream = VideoStream::new(pool)?;
    let stream = Data::new(Mutex::new(stream));

    HttpServer::new(move ||
            App::new()
                .register_data(stream.clone())
                .route("/api/stream", web::get().to(get_stream))
                .route("/api/stream", web::patch().to(patch_stream))
        )
        .bind("127.0.0.1:9350")?
        .run()?;

    Ok(())
}
