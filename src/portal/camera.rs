//! Camera management

use diesel::prelude::*;
use lunacam::Result;
use lunacam::api::{CameraSettings, Orientation};
use lunacam::db::DatabaseContext;
use lunacam::db::schema::cameras;
use log::{debug, info, trace};
use serde::{Serialize};


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
struct CameraRow {
    id: i32,
    friendly_name: String,
    hostname: String,
    device_key: String,
    enabled: bool,
    orientation: Orientation,
}


#[derive(Serialize)]
pub struct Camera<'a, M> {
    #[serde(flatten)]
    row: CameraRow,
    #[serde(skip)]
    manager: &'a M,
}


impl<'a, M> Camera<'a, M>
where M: CameraManager
{
    /// Deletes this camera from the database
    pub fn delete(self) -> Result<()> {

        debug!("deleting camera {}", self.row.id);
        let conn = self.manager.conn()?;
        diesel::delete(&self.row)
            .execute(&conn)?;

        Ok(())
    }

    /// Gets the ID of this camera
    pub fn id(&self) -> i32 {
        self.row.id
    }

    /// Updates camera settings
    pub fn update(&mut self, settings: CameraSettings) -> Result<()> {

        assert!(settings.id.is_none());

        let mut do_connect = false;
        let mut do_update = false;
        let mut do_save = false;

        if let Some(friendly_name) = settings.friendly_name {
            if self.row.friendly_name != friendly_name {
                self.row.friendly_name = friendly_name;
                do_save = true;
            }
        }

        if let Some(enabled) = settings.enabled {
            if self.row.enabled != enabled {
                self.row.enabled = enabled;
                do_update = true;
                do_save = true;
            }
        }

        if let Some(orientation) = settings.orientation {
            if self.row.orientation != orientation {
                self.row.orientation = orientation;
                do_update = true;
                do_save = true;
            }
        }

        if let Some(hostname) = settings.hostname {
            if self.row.hostname != hostname {
                self.row.hostname = hostname;
                do_connect = true;
                do_save = true;
            }
        }

        if let Some(device_key) = settings.device_key {
            if self.row.device_key != device_key {
                self.row.device_key = device_key;
                do_connect = true;
                do_save = true;
            }
        }

        if do_connect {
            // TODO: connect to camera
        }

        if do_update {
            info!("reconfiguring {}", self.row.hostname);
            // TODO: send PATCH to camera host
        }

        if do_save {
            debug!("saving changes to camera {}", self.row.id);
            let conn = self.manager.conn()?;
            diesel::update(&self.row)
                .set(&self.row)
                .execute(&conn)?;
        }

        Ok(())
    }
}

impl<'a, M> Into<CameraSettings> for Camera<'a, M> {
    fn into(self) -> CameraSettings {
        CameraSettings {
            enabled: Some(self.row.enabled),
            hostname: Some(self.row.hostname),
            id: Some(self.row.id),
            device_key: None,
            friendly_name: Some(self.row.friendly_name),
            orientation: Some(self.row.orientation),
        }
    }
}


pub trait CameraManager: DatabaseContext + Sized {

    /// Creates a new camera in the database
    fn create_camera(&self, hostname: String, key: String) -> Result<Camera<Self>> {

        let conn = self.conn()?;

        let new_cam = NewCamera {
            hostname: hostname.clone(),
            device_key: key,
            friendly_name: hostname,
        };

        debug!("adding new camera to database");
        diesel::insert_into(cameras::table)
            .values(new_cam)
            .execute(&conn)?;

        // Yuck. But this is the only option when using SQLite.
        // https://github.com/diesel-rs/diesel/issues/376
        // TODO: this technically isn't thread-safe
        let cam_row = cameras::table.order(cameras::id.desc())
            .first(&conn)?;

        Ok(Camera {
            row: cam_row,
            manager: self,
        })
    }

    /// Gets the specified camera from the database
    fn get_camera(&self, id: i32) -> Result<Camera<Self>> {

        trace!("retrieving camera {} from database", id);
        let conn = self.conn()?;
        let camera = cameras::table.find(id)
            .get_result(&conn)?;

        Ok(Camera {
            row: camera,
            manager: self,
        })
    }

    /// Gets all cameras from the database
    fn get_cameras(&self) -> Result<Vec<Camera<Self>>> {

        trace!("retrieving all cameras from database");
        let conn = self.conn()?;
        let cameras = cameras::table.load(&conn)?
            .into_iter()
            .map(|c| Camera {
                row: c,
                manager: self,
            })
            .collect();

        Ok(cameras)
    }
}

impl<T: DatabaseContext + Sized> CameraManager for T {}
