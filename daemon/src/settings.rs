//! Static application settings

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use log::{trace};
use lunacam::Result;
use lunacam::db::schema::settings;
use serde::Serialize;
use serde::de::DeserializeOwned;


#[derive(AsChangeset, Identifiable, Insertable, Queryable)]
#[table_name = "settings"]
#[primary_key(name)]
struct RawSetting {
    name: String,
    value: String,
}


pub fn get<T: DeserializeOwned>(name: &str, db: &SqliteConnection) -> Result<Option<T>> {

    trace!("retrieving \"{}\" setting from database", name);
    let query_result = settings::table.find(name)
        .get_result(db);

    let raw: RawSetting = match query_result {

        // Setting has never been set, return None
        Err(diesel::result::Error::NotFound) => {
            trace!("could not find setting \"{}\"", name);
            return Ok(None);
        },

        // Found setting
        Ok(raw) => raw,

        // Unexpected error
        Err(err) => {
            return Err(err.into());
        },
    };

    Ok(Some(serde_json::from_str(&raw.value)?))
}


pub fn set<T>(name: &str, value: &T, db: &SqliteConnection) -> Result<()>
where T: DeserializeOwned + Serialize
{

    let raw = RawSetting {
        name: name.into(),
        value: serde_json::to_string(value)?,
    };

    trace!("storing \"{}\" setting in database", name);

    // Diesel does not yet support upsert for SQLite
    // https://github.com/diesel-rs/diesel/pull/1884
    if get::<T>(name, db)?.is_none() {
        diesel::insert_into(settings::table)
            .values(&raw)
            .execute(db)?;
    } else {
        diesel::update(&raw)
            .set(&raw)
            .execute(db)?;
    }

    Ok(())
}
