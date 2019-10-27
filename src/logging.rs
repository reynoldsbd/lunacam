//! Logging utilities


use env_logger::Env;


/// Initializes environment-based logging provider
pub fn init() {

    let env = Env::default()
        .filter_or("LC_LOG", "info")
        .write_style("LC_LOG_STYLE");

    env_logger::init_from_env(env);
}
