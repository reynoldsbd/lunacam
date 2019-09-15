use std::sync::Mutex;
use actix_web::{App, HttpServer};
use actix_web::web::{self, Data};
use lunacam::Result;
use lunacam::db;
use lunacam::logging;

mod api;
mod stream;

use stream::VideoStream;


fn main() -> Result<()> {

    logging::init();

    let pool = db::connect()?;
    let stream = Data::new(Mutex::new(VideoStream::new(pool)?));

    HttpServer::new(move ||
            App::new()
                .register_data(stream.clone())
                .service(web::scope("/api").configure(api::configure))
        )
        .bind("127.0.0.1:9350")?
        .run()?;

    Ok(())
}
