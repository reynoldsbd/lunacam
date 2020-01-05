//! Manages the Nginx server behind which the LunaCam application and HLS
//! streams are reverse-proxied

use std::env;
use std::process::{Stdio};

use tokio::fs;
use tokio::process::Command;

use log::{debug, trace, warn};

use crate::error::Result;


/// Reloads server configuration
pub async fn reload() -> Result<()> {

    debug!("reloading proxy");

    if cfg!(windows) {
        warn!("skipping proxy reload on Windows host");
        return Ok(())
    }

    let status = Command::new("/usr/bin/sudo")
        .args(&["-n", "/usr/bin/systemctl", "reload", "nginx.service"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;

    if !status.success() {
        warn!("failed to reload nginx");
    }

    Ok(())
}


/// Retrieves the proxy configuration directory, creating it if it does not yet
/// exist.
pub async fn config_dir() -> Result<String> {

    trace!("identifying proxy config directory");

    let rt_dir = env::var("RUNTIME_DIRECTORY")?;
    let cfg_dir = format!("{}/nginx", rt_dir);

    if fs::metadata(&cfg_dir).await.is_err() {
        debug!("creating proxy config directory {}", cfg_dir);
        fs::create_dir_all(&cfg_dir)
            .await?;
    }

    Ok(cfg_dir)
}
