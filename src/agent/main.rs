use std::mem;
use std::sync::RwLock;

use actix_web::{App, HttpServer};
use actix_web::web::{self, Data};

use lunacam::error::Result;
use lunacam::db;
use lunacam::logging;
use lunacam::stream;


fn main() -> Result<()> {

    logging::init();

    let pool = db::connect()?;

    let conn = pool.get()?;
    let stream = stream::initialize(&conn)?;
    mem::drop(conn);

    let pool = Data::new(pool);
    let stream = Data::new(RwLock::new(stream));

    HttpServer::new(move ||
            App::new()
                .register_data(pool.clone())
                .register_data(stream.clone())
                .service(web::scope("/api").configure(stream::configure_api))
        )
        .bind("127.0.0.1:9350")?
        .run()?;

    Ok(())
}
