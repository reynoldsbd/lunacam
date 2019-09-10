use actix_web::{App, HttpServer};
use actix_web::web;
use env_logger::Env;
use lunacam::Result;
use lunacam::db;
use tokio::runtime::{Runtime};

mod api;
mod transcoder;


fn main() -> Result<()> {

    let env = Env::default()
        .filter_or("LC_LOG", "info")
        .write_style("LC_LOG_STYLE");
    env_logger::init_from_env(env);

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
