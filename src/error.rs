//! Error handling used throughout LunaCam

use std::env::VarError;
use std::io;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use diesel_migrations::RunMigrationsError;
use tokio::executor::SpawnError;


/// Error type generated by LunaCam
#[derive(Debug, Display, From)]
pub enum Error {

    /// Error generated by database query
    Database(diesel::result::Error),

    /// Failure establishing connection to database
    DatabaseConnection(diesel::result::ConnectionError),

    /// General I/O error
    Io(io::Error),

    /// Failure serializing or deserializing JSON
    Json(serde_json::Error),

    /// Error running database migrations
    Migration(RunMigrationsError),

    /// Error generated by database connection pool
    Pool(diesel::r2d2::PoolError),

    /// Failed to spawn a task
    Spawn(SpawnError),

    /// Error generated by Tera
    Tera(tera::Error),

    /// Error reading an environment variable
    Var(VarError),

    /// Error produced by a failed web request
    #[display(fmt = "{}", _1)]
    Web(StatusCode, &'static str),
}

impl ResponseError for Error {

    // TODO: maybe render something nice
}


/// Result type generated by LunaCam
pub type Result<T> = std::result::Result<T, Error>;
