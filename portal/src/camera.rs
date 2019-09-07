//! Camera management

use diesel::prelude::*;
use lc_api::{CameraSettings, Orientation};
use log::{debug, info, trace};
use serde::{Serialize};
use crate::schema::cameras;


/// Information needed to create a new camera
#[derive(Insertable)]
#[table_name = "cameras"]
struct NewCamera {
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
    id: i32,
    friendly_name: String,
    hostname: String,
    device_key: String,
    enabled: bool,
    orientation: Orientation,
}

impl Camera
{
    /// Creates a new camera in the database
    pub fn create(
        hostname: String,
        device_key: String,
        db: &SqliteConnection,
    ) -> QueryResult<Self>
    {
        let camera = NewCamera {
            hostname: hostname.clone(),
            device_key: device_key,
            friendly_name: hostname,
        };

        debug!("adding new camera to database");
        diesel::insert_into(cameras::table)
            .values(camera)
            .execute(db)?;

        // Yuck. But this is the only option when using SQLite.
        // https://github.com/diesel-rs/diesel/issues/376
        // TODO: this technically isn't thread-safe
        trace!("retrieving new camera from database");
        cameras::table.order(cameras::id.desc())
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

    /// Updates camera settings
    pub fn update(&mut self, settings: CameraSettings, db: &SqliteConnection) -> QueryResult<()> {

        assert!(settings.id.is_none());

        let mut do_connect = false;
        let mut do_update = false;
        let mut do_save = false;

        if let Some(friendly_name) = settings.friendly_name {
            if self.friendly_name != friendly_name {
                self.friendly_name = friendly_name;
                do_save = true;
            }
        }

        if let Some(enabled) = settings.enabled {
            if self.enabled != enabled {
                self.enabled = enabled;
                do_update = true;
                do_save = true;
            }
        }

        if let Some(orientation) = settings.orientation {
            if self.orientation != orientation {
                self.orientation = orientation;
                do_update = true;
                do_save = true;
            }
        }

        if let Some(hostname) = settings.hostname {
            if self.hostname != hostname {
                self.hostname = hostname;
                do_connect = true;
                do_save = true;
            }
        }

        if let Some(device_key) = settings.device_key {
            if self.device_key != device_key {
                self.device_key = device_key;
                do_connect = true;
                do_save = true;
            }
        }

        if do_connect {
            // TODO: connect to camera
        }

        if do_update {
            info!("reconfiguring {}", self.hostname);
            // TODO: send PATCH to camera host
        }

        if do_save {
            debug!("saving changes to camera {}", self.id);
            diesel::update(self as &_)
                .set(self as &_)
                .execute(db)?;
        }

        Ok(())
    }

    /// Gets the ID of this camera
    pub fn id(&self) -> i32 {
        self.id
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
