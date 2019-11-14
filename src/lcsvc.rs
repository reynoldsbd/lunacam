use std::env;
use std::mem;
use std::sync::RwLock;

use actix_web::web::{Data};
use tera::Tera;

use lunacam::cameras;
use lunacam::db;
use lunacam::error::Result;
use lunacam::logging;
use lunacam::stream;
use lunacam::users;


#[cfg(not(any(feature = "portal", feature = "stream-api")))]
compile_error!("invalid feature selection");


fn main() -> Result<()> {

    logging::init();

    // Prepare application resources

    #[cfg(debug_assertions)]
    let static_dir = env::var("LC_STATIC")?;

    let template_dir = env::var("LC_TEMPLATES")?;
    let template_dir = format!("{}/**/*", template_dir);
    let templates = Tera::new(&template_dir)?;

    let pool = db::connect()?;
    let conn = pool.get()?;

    #[cfg(feature = "portal")]
    {
        cameras::initialize_proxy_config(&conn, &templates)?;
        users::maybe_create_default_user(&conn)?;
    }

    #[cfg(feature = "stream")]
    let stream = Data::new(RwLock::new(stream::initialize(&conn)?));

    mem::drop(conn);

    println!("hello, world");

    Ok(())
}
