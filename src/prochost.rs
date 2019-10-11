//! Child process lifecycle management

use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use log::{debug, error, trace, warn};
use crate::error::Result;
use crate::do_lock;


/// Internal state of the process host
struct HostState {
    cmd: Command,
    child: Option<Child>,
}


/// Watchdog tick interval
const WDG_TICK_SECONDS: u64 = 2;


/// Periodically checks that the child process is still running and restarts it if necessary
fn host_wdg(hi: &Mutex<HostState>) {

    let tick_duration = Duration::from_secs(WDG_TICK_SECONDS);

    loop {
        trace!("watchdog tick");

        // Tick body scoped to ensure lock is not held
        {
            let mut hi = do_lock!(hi);

            let wait_res = if let Some(ref mut child) = hi.child {
                child.try_wait()
            } else {
                trace!("host has been stopped");
                break;
            };

            match wait_res {

                // Process is still running, everything OK
                Ok(None) => (),

                // Child process no longer running
                Ok(Some(status)) => {
                    warn!("child process exited unexpectedly with status {}", status);
                    debug!("restarting child process");
                    match hi.cmd.spawn() {
                        Ok(child) => {
                            hi.child.replace(child);
                        },
                        Err(err) => {
                            error!("failed to restart child process: {}", err);
                            break;
                        }
                    }
                },

                // Error checking status
                Err(err) => {
                    error!("failed to check child process status: {}", err);
                    break;
                },
            }
        }

        thread::sleep(tick_duration);
    }

    debug!("watchdog exiting");
}


/// Hosts and monitors a child process
///
/// `ProcHost` is a wrapper around the standard library's `Child` that watches the child process and
/// restarts it as necessary.
pub struct ProcHost(Arc<Mutex<HostState>>);

impl ProcHost {

    /// Creates a new `ProcHost`
    ///
    /// Hosted process is started/restarted according to `cmd`
    pub fn new(cmd: Command) -> Self {

        Self(Arc::new(Mutex::new(HostState {
            cmd,
            child: None,
        })))
    }

    /// Starts the child process
    ///
    /// If child is already running, no action is taken.
    pub fn start(&mut self) -> Result<()> {

        let wdg_hi = self.0.clone();
        let mut hi = do_lock!(self.0);

        if hi.child.is_some() {
            trace!("start called, but child process already running");
        } else {
            debug!("starting child process");
            let child = hi.cmd.spawn()?;
            hi.child.replace(child);
        }

        thread::spawn(move || host_wdg(&wdg_hi));

        Ok(())
    }

    /// Stops the child process
    ///
    /// If child is not currently running, no action is taken.
    pub fn stop(&mut self) -> Result<()> {

        let mut hi = do_lock!(self.0);

        if let Some(mut child) = hi.child.take() {
            debug!("stopping child process");
            child.kill()?;
        } else {
            trace!("stop called, but child process not running");
        }

        Ok(())
    }
}

/// Child process is terminated when `ProcHost` is dropped
impl Drop for ProcHost {
    fn drop(&mut self) {
        if let Err(err) = self.stop() {
            error!("failed to stop child process after dropping: {}", err);
        }
    }
}
