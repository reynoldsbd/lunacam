//! Child process lifecycle management

use std::mem;
use std::sync::Arc;

use futures::FutureExt;

use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::sync::oneshot::{self, Sender, Receiver};
use tokio::task::{self, JoinHandle};

use log::{debug, error, trace, warn};

use crate::error::Result;


/// Monitors and restarts `child` until signaled to stop via `rx`
#[allow(clippy::unnecessary_mut_passed)]
async fn watchdog(child: Child, cmd: Arc<Mutex<Command>>, rx: Receiver<()>) {

    // Prep futures for use with select!
    let child = child.fuse();
    let rx = rx.fuse();
    futures::pin_mut!(child, rx);

    trace!("watchdog started");

    loop {

        futures::select! {

            status = child => match status {

                Ok(status) => {

                    warn!("child process exited with status {}", status);

                    debug!("restarting child process");
                    match cmd.lock().await.spawn() {

                        Ok(c) => {
                            let c = c.fuse();
                            *child = c;
                        },

                        Err(err) => {
                            error!("failed to restart child process: {}", err);
                            break;
                        },
                    }
                },

                Err(err) => {

                    error!("failed to check child process status: {}", err);
                    break;
                }
            },

            // Received signal from Watchdog::stop
            _ = rx => break,
        }
    }

    // Can't call Child::kill, because child has been fused to facilitate the
    // above select! construct. Instead, we rely on ProcHost::new to call
    // Command::kill_on_drop, then simply drop the child.
    debug!("killing child process");
    mem::drop(child);

    trace!("watchdog exiting");
}


/// Hosts and monitors a child process
///
/// When the host is placed into the running state, the specified child process
/// is started and checked periodically to ensure it is still running. If the
/// process exits for any reason other than being terminated via the host (i.e.
/// by calling `ProcHost::stop`), it is automatically restarted.
pub struct ProcHost {
    cmd: Arc<Mutex<Command>>,
    wdg: Option<(JoinHandle<()>, Sender<()>)>,
}

impl ProcHost {

    /// Creates a new `ProcHost`
    ///
    /// Hosted process is started/restarted according to `cmd`
    pub fn new<C: Into<Command>>(cmd: C) -> Self {

        let mut cmd = cmd.into();

        // Ensures child is properly killed when watchdog exits
        cmd.kill_on_drop(true);

        Self {
            cmd: Arc::new(Mutex::new(cmd)),
            wdg: None,
        }
    }

    /// Starts the child process
    ///
    /// If child is already running, no action is taken.
    pub async fn start(&mut self) -> Result<()> {

        if self.wdg.is_some() {

            trace!("start called, but child process is already running");

        } else {

            debug!("starting child process");
            let child = self.cmd.lock()
                .await
                .spawn()?;

            trace!("starting watchdog");
            let (tx, rx) = oneshot::channel();
            let handle = task::spawn(watchdog(child, self.cmd.clone(), rx));

            self.wdg.replace((handle, tx));
        }

        Ok(())
    }

    /// Stops the child process
    ///
    /// If child is not currently running, no action is taken.
    pub async fn stop(&mut self) -> Result<()> {

        if let Some((handle, tx)) = self.wdg.take() {

            trace!("stopping watchdog");
            if tx.send(()).is_err() {
                error!("failed to signal watchdog");
            } else {
                handle.await?;
            }

        } else {

            trace!("stop called, but child process not running");
        }

        Ok(())
    }

    /// Returns whether this host is currently in the running state
    pub fn running(&self) -> bool {

        self.wdg.is_some()
    }
}

/// Child process is automatically terminated when `ProcHost` is dropped
impl Drop for ProcHost {

    fn drop(&mut self) {

        if let Some((_, tx)) = self.wdg.take() {

            trace!("stopping watchdog");
            if tx.send(()).is_err() {
                error!("failed to signal watchdog");
            }
        }
    }
}
