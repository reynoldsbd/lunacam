//! Manages lifecycle of child processes


use std::process::{Child, Command, Stdio};
use std::time::Duration;
use actix::{Actor, AsyncContext, Context, Handler, Message, SpawnHandle};
use log::{error, trace, warn};


/// Interval between `ProcessHost` watchdog ticks
const WDG_INTERVAL: Duration = Duration::from_secs(5);


/// Hosts a child process
///
/// `ProcessHost` is an actor that manages the lifecycle of a child process. The process can be
/// started or stopped by sending messages to the host. Additionally, a watchdog is used to restart
/// the child if it terminates unexpectedly.
///
/// Interaction with the child using stdio is not supported. The `Command` used to spawn the child
/// will be unconditionally modified to ignore stdin, stdout, and stderr.
pub struct ProcessHost
{
    child: Option<Child>,
    cmd: Command,
    wdg: Option<SpawnHandle>,
}

impl ProcessHost
{
    /// Creates a new `ProcessHost`, but does not start the child process
    pub fn new(mut cmd: Command) -> Self
    {
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        ProcessHost {
            child: None,
            cmd: cmd,
            wdg: None,
        }
    }

    /// Starts the child process
    fn start(&mut self, ctx: &mut Context<Self>)
    {
        trace!("starting child process");
        match self.cmd.spawn() {
            Ok(child) => self.child = Some(child),
            Err(err) => {
                error!("Failed to start child process: {}", err);
                return;
            }
        }

        if self.wdg.is_none() {
            trace!("scheduling watchdog");
            let wdg = ctx.run_interval(WDG_INTERVAL, ProcessHost::watchdog);
            self.wdg = Some(wdg);
        }
    }

    /// Stops the child process
    fn stop(&mut self, ctx: &mut Context<Self>)
    {
        if let Some(mut child) = self.child.take() {
            trace!("stopping child process");
            if let Err(err) = child.kill() {
                error!("Failed to stop child process: {}", err);
            }

        } else {
            warn!("Attempted to stop child process that is not running");
        }

        if let Some(wdg) = self.wdg.take() {
            trace!("cancelling watchdog");
            ctx.cancel_future(wdg);

        } else {
            warn!("Could not cancel watchdog");
        }
    }

    /// Monitors and restarts the child process
    fn watchdog(&mut self, ctx: &mut Context<Self>)
    {
        trace!("process host watchdog tick");

        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(status)) => {
                    warn!("Child process exited unexpectedly ({}). Restarting...", status);
                    self.start(ctx);
                },
                Ok(None) => (),
                Err(err) => error!("Failed to check child process status: {}", err),
            }
        } else {
            warn!("Orphaned watchdog");
        }
    }
}

impl Actor for ProcessHost
{
    type Context = Context<Self>;

    fn stopped(&mut self,  ctx: &mut Self::Context)
    {
        self.stop(ctx);
        trace!("stopped process host");
    }
}


/// Instructs a `ProcessHost` to start the hosted process
pub struct StartProcess;

impl Message for StartProcess
{
    type Result = ();
}

impl Handler<StartProcess> for ProcessHost
{
    type Result = ();

    fn handle(&mut self, _: StartProcess, ctx: &mut Context<Self>) -> Self::Result
    {
        self.start(ctx);
    }
}


/// Instructs a `ProcessHost` to stop the hosted process
pub struct StopProcess;

impl Message for StopProcess
{
    type Result = ();
}

impl Handler<StopProcess> for ProcessHost
{
    type Result = ();

    fn handle(&mut self, _: StopProcess, ctx: &mut Context<Self>) -> Self::Result
    {
        self.stop(ctx);
    }
}
