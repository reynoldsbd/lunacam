//! Camera management

use std::env;
use std::fs;
use std::process::{Command, Stdio};

use diesel::prelude::*;
use log::{debug, error, info, trace, warn};
use serde::{Serialize};
use tera::{Context, Tera};

use lunacam::allow_err;
use lunacam::Result;
use lunacam::api::{CameraSettings, Orientation};
use lunacam::db::DatabaseContext;
use lunacam::db::schema::cameras;


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


/// Reloads proxy server configuration
fn reload_proxy() -> Result<()> {

    debug!("reloading nginx");

    let status = Command::new("/usr/bin/sudo")
        .args(&["-n", "/usr/bin/systemctl", "reload", "nginx.service"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        warn!("failed to reload nginx");
    }

    Ok(())
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
    /// Gets path of the proxy configuration file for this camera
    fn configuration_path(&self) -> Result<impl AsRef<std::path::Path>> {

        let state_dir = env::var("STATE_DIRECTORY")?;
        let path = format!("{}/nginx/proxy-{}.config", state_dir, self.row.id);

        Ok(path)
    }

    /// Writes or removes Nginx configuration snippet for this camera
    fn configure_proxy(&self, reload: bool) -> Result<()> {

        let config_path = self.configuration_path()?;
        let mut needs_reload = false;

        if self.row.enabled {
            debug!("writing proxy configuration for camera {}", self.row.id);
            let mut context = Context::new();
            context.insert("camera", self);
            let config = self.manager.as_ref().render("proxy.config", context)?;
            fs::write(&config_path, config)?;
            needs_reload = true;

        } else if fs::metadata(&config_path).is_ok() {
            debug!("clearing proxy configuration for camera {}", self.row.id);
            fs::remove_file(&config_path)?;
            needs_reload = true;
        }

        if reload && needs_reload {
            reload_proxy()?;
        }

        Ok(())
    }

    /// Deletes this camera from the database
    pub fn delete(self) -> Result<()> {

        debug!("deleting camera {}", self.row.id);
        let conn = self.manager.conn()?;
        diesel::delete(&self.row)
            .execute(&conn)?;

        if self.row.enabled {
            debug!("deleting proxy configuration for camera {}", self.row.id);
            fs::remove_file(self.configuration_path()?)?;
            reload_proxy()?;
        }

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
            allow_err!(
                self.configure_proxy(true),
                "failed to reconfigure proxy for camera {}",
                self.row.id
            );
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


pub trait CameraManager: DatabaseContext + AsRef<Tera> + Sized {

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
        let cam = Camera {
            row: cam_row,
            manager: self,
        };

        // TODO: connect to camera

        allow_err!(
            cam.configure_proxy(true),
            "failed to configure proxy for camera {}",
            cam.row.id
        );

        Ok(cam)
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

    /// Ensures reverse proxy is properly configured
    fn initialize_proxy(&self) -> Result<()> {

        let state_dir = env::var("STATE_DIRECTORY")?;
        fs::create_dir_all(format!("{}/nginx", state_dir))?;

        for camera in self.get_cameras()? {
            camera.configure_proxy(false)?;
        }

        reload_proxy()?;

        Ok(())
    }
}

impl<T: DatabaseContext + AsRef<Tera> + Sized> CameraManager for T {}
