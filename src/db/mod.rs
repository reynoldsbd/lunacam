//! Database connectivity


use std::borrow::Borrow;
use std::env;
use diesel::r2d2::{self, ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::embed_migrations;
use crate::error::Result;


pub mod schema;


/// Connection pool type used throughout LunaCam
pub type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;


/// Pooled connection type used throughout LunaCam
pub type PooledConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;


embed_migrations!();


/// Connects to the LunaCam database
///
/// Database is created and initialized if necessary.
pub fn connect() -> Result<ConnectionPool> {

    let state_dir = env::var("STATE_DIRECTORY")?;
    let db_url = format!("{}/lunacam.db", state_dir);
    let pool = Pool::new(ConnectionManager::new(db_url))?;

    // Ensure database is initialized
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
