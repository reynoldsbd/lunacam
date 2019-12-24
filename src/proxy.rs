//! Manages the Nginx server behind which the LunaCam application and HLS
//! streams are reverse-proxied

use std::env;
use std::fs;
use std::process::{Command, Stdio};

use log::{debug, trace, warn};

use crate::error::Result;


/// Ensures the proxy config directory exists
pub fn init() -> Result<()> {

    trace!("identifying proxy config directory");
    let state_dir = env::var("STATE_DIRECTORY")?;

    let config_dir = format!("{}/nginx", state_dir);
    trace!("checking for presence of proxy config directory {}", config_dir);
    if fs::metadata(&config_dir).is_err() {

        debug!("creating proxy config dir {}", config_dir);
        fs::create_dir_all(&config_dir)?;
    }

    Ok(())
}


/// Reloads server configuration
pub fn reload() -> Result<()> {

    debug!("reloading proxy");

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
