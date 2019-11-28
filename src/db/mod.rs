//! Database connectivity


use std::borrow::Borrow;
use std::env::{self, VarError};

use diesel::r2d2::{self, ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::embed_migrations;
use log::{debug, trace};

use crate::error::Result;


pub mod schema;


/// Connection pool type used throughout LunaCam
pub type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;


/// Pooled connection type used throughout LunaCam
pub type PooledConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;


embed_migrations!();


/// Connects to and initializes the LunaCam database
///
/// Database file is named *lunacam.db* and placed under the directory given by
/// the STATE_DIRECTORY environment variable. If that variable is not present
/// and this program is compiled in debug mode, the database file is placed in
/// the current working directory.
pub fn connect() -> Result<ConnectionPool> {

    trace!("identifying state directory");
    let db_dir = match env::var("STATE_DIRECTORY") {
        Ok(dir) => dir,
        #[cfg(debug_assertions)]
        Err(VarError::NotPresent) => String::from("."),
        Err(err) => return Err(err.into()),
    };

    let db_url = format!("{}/lunacam.db", db_dir);
    debug!("connecting to database at {}", db_url);
    let pool = Pool::new(ConnectionManager::new(db_url))?;

    debug!("running migrations if necessary");
    let conn = pool.get()?;
    embedded_migrations::run(&conn)?;

    Ok(pool)
}


/// Provides access to the application database
pub trait DatabaseContext {

    /// Gets a pooled database connection
    fn conn(&self) -> Result<PooledConnection>;
}

impl<T> DatabaseContext for T
where T: Borrow<ConnectionPool>
{
    fn conn(&self) -> Result<PooledConnection> {
        Ok(self.borrow().get()?)
    }
}
