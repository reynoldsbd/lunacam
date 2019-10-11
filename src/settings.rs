//! Global application settings management


use diesel::prelude::*;
use diesel::result::Error as DieselError;
use log::trace;
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::db::DatabaseContext;
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


/// Provides access to global application settings
#[allow(clippy::module_name_repetitions)] // "Provider" is too generic
pub trait SettingsProvider: DatabaseContext {

    /// Retrieves the specified setting from the database
    ///
    /// If the setting has never been stored in the database before, `None` is returned. If the
    /// stored data cannot be deserialized as `T`, an error is propagated from `serde_json`.
    fn get_setting<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>> {

        trace!("retrieving setting \"{}\" from database", name);
        let conn = self.conn()?;
        let res = settings::table.find(name)
            .get_result(&conn);

        let setting: Setting = match res {

            // Setting has never been set, return None
            Err(DieselError::NotFound) => {
                trace!("could not find setting \"{}\"", name);
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
    fn set_setting<T>(&self, name: &str, value: &T) -> Result<()>
    where T: DeserializeOwned + Serialize
    {

        let setting = Setting {
            name: name.into(),
            value: serde_json::to_string(value)?,
        };

        trace!("storing setting \"{}\" to database", name);

        // Diesel does not yet support upsert for SQLite
        // https://github.com/diesel-rs/diesel/pull/1884
        let conn = self.conn()?;
        if self.get_setting::<T>(name)?.is_none() {
            diesel::insert_into(settings::table)
                .values(&setting)
                .execute(&conn)?;
        } else {
            diesel::update(&setting)
                .set(&setting)
                .execute(&conn)?;
        }

        Ok(())
    }
}

impl<T: DatabaseContext> SettingsProvider for T {}
