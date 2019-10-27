//! Global application settings management


use diesel::prelude::*;
use diesel::result::Error as DieselError;
use log::{debug, trace};
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::db::PooledConnection;
use crate::db::schema::settings;
use crate::error::Result;


/// Serialized form of a setting
#[derive(AsChangeset, Identifiable, Insertable, Queryable)]
#[table_name = "settings"]
#[primary_key(name)]
struct Setting {
    name: String,
    value: String,
}


/// Retrieves the specified setting from the database
///
/// If the setting has never been stored in the database before, `None` is returned. If the stored
/// data cannot be deserialized as `T`, an error is propagated from `serde_json`.
pub fn get<T: DeserializeOwned>(name: &str, conn: &PooledConnection) -> Result<Option<T>> {

    debug!("retrieving setting {} from database", name);
    let setting = settings::table.find(name)
        .get_result(conn);

    let setting: Setting = match setting {

        // Setting has never been set
        Err(DieselError::NotFound) => {
            trace!("could not find setting {}", name);
            return Ok(None);
        },

        // Found setting
        Ok(setting) => setting,

        // Unexpected error
        Err(err) => return Err(err.into()),
    };

    Ok(Some(serde_json::from_str(&setting.value)?))
}


/// Stores the given setting to the database
pub fn set<T>(name: &str, value: &T, conn: &PooledConnection) -> Result<()>
where T: DeserializeOwned + Serialize
{
    let setting = Setting {
        name: name.into(),
        value: serde_json::to_string(value)?,
    };

    // Diesel does not yet support upsert for SQLite
    // https://github.com/diesel-rs/diesel/pull/1884
    if get::<T>(name, conn)?.is_none() {
        debug!("storing new setting {} in database", name);
        diesel::insert_into(settings::table)
            .values(&setting)
            .execute(conn)?;
    } else {
        debug!("updating setting {} in database", name);
        diesel::update(&setting)
            .set(&setting)
            .execute(conn)?;
    }

    Ok(())
}
