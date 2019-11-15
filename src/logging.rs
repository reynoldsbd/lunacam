//! Logging utilities


use env_logger::Env;


#[cfg(debug_assertions)]
const DEFAULT_FILTER: &str = "info,lunacam=debug";
#[cfg(not(debug_assertions))]
const DEFAULT_FILTER: &str = "info";


/// Initializes environment-based logging provider
pub fn init() {

    let env = Env::default()
        .filter_or("LC_LOG", DEFAULT_FILTER)
        .write_style("LC_LOG_STYLE");

    env_logger::init_from_env(env);
}
