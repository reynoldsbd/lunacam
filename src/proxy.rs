//! Manages the Nginx server behind which the LunaCam application is
//! reverse-proxied

use std::process::{Command, Stdio};

use log::{debug, warn};

use crate::error::Result;


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
