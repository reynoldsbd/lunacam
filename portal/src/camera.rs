//! Camera management

use diesel::prelude::*;
use lc_api::{CameraSettings, Orientation};
use log::{debug, trace};
use serde::{Serialize};
use crate::schema::cameras;


/// Information needed to create a new camera
#[derive(Insertable)]
#[table_name = "cameras"]
pub struct NewCamera {
    pub hostname: String,
    pub device_key: String,
    pub friendly_name: String,
}


/// Represents a streaming camera controlled by this server
#[derive(Serialize)]
#[derive(AsChangeset, Identifiable, Insertable, Queryable)]
#[table_name = "cameras"]
pub struct Camera
{
    pub id: i32,
    pub friendly_name: String,
    pub hostname: String,
    pub device_key: String,
    pub enabled: bool,
    pub orientation: Orientation,
}

impl Camera
{
    /// Creates a new camera in the database
    pub fn create(camera: NewCamera, db: &SqliteConnection) -> QueryResult<Self>
    {
        use crate::schema::cameras::dsl::*;

        debug!("adding new camera to database");
        diesel::insert_into(cameras)
            .values(camera)
            .execute(db)?;

        // Yuck. But this is the only option when using SQLite.
        // https://github.com/diesel-rs/diesel/issues/376
        // TODO: this technically isn't thread-safe
        trace!("retrieving new camera from database");
        cameras.order(id.desc())
            .first(db)
    }

    /// Deletes this camera from the database
    pub fn delete(self, db: &SqliteConnection) -> QueryResult<()>
    {
        debug!("deleting camera {}", self.id);
        diesel::delete(&self)
            .execute(db)?;

        Ok(())
    }

    /// Gets all cameras from the database
    pub fn get_all(db: &SqliteConnection) -> QueryResult<Vec<Self>>
    {
        use crate::schema::cameras::dsl::*;

        trace!("retrieving all cameras from database");
        cameras.load(db)
    }

    /// Gets the specified camera from the database
    pub fn get(cam_id: i32, db: &SqliteConnection) -> QueryResult<Self>
    {
        use crate::schema::cameras::dsl::*;

        trace!("retrieving camera {} from database", cam_id);
        cameras.find(cam_id)
            .get_result(db)
    }

    /// Saves any changes to this camera to the database
    pub fn save(&self, db: &SqliteConnection) -> QueryResult<()>
    {
        debug!("saving camera {} to database", self.id);
        diesel::update(self)
            .set(self)
            .execute(db)?;

        Ok(())
    }
}

impl Into<CameraSettings> for Camera {
    fn into(self) -> CameraSettings {
        CameraSettings {
            enabled: Some(self.enabled),
            hostname: Some(self.hostname),
            id: Some(self.id),
            device_key: None,
            friendly_name: Some(self.friendly_name),
            orientation: Some(self.orientation),
        }
    }
}
