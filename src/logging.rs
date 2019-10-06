//! Logging utilities


use env_logger::Env;


/// Initializes environment-based logging provider
pub fn init() {

    let env = Env::default()
        .filter_or("LC_LOG", "info")
        .write_style("LC_LOG_STYLE");

    env_logger::init_from_env(env);
}


/// Logs an error if the given `Result` is `Err`
#[macro_export]
macro_rules! allow_err {
    ($res:expr, $fmt:expr $(, $args:expr)*) => ({
        use log::error;
        if let Err(err) = $res {
            let msg = format!($fmt, $($args),*);
            error!("{}: {}", msg, err);
        }
    })
}
