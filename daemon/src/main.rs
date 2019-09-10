use actix_web::{App, HttpServer};
use actix_web::web;
use lunacam::Result;
use lunacam::db;
use lunacam::logging;
use tokio::runtime::{Runtime};

mod api;
mod transcoder;


fn main() -> Result<()> {

    logging::init();

    let pool = db::connect()?;

    let rt = Runtime::new()?;

    transcoder::initialize(Box::new(rt.executor()), pool)?;

    HttpServer::new(||
            App::new()
                .service(web::scope("/api").configure(api::configure))
        )
        .bind("127.0.0.1:9350")?
        .run()?;

    Ok(())
}
