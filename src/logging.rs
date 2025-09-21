use colored::Colorize;
use std::time::{Instant};

/// Log a success message in green
pub fn success(msg: &str) {
    log::info!("{}", msg.green());
}

pub fn info(msg: &str)
{
    log::info!("{msg}");
}

/// Log an error message in red
pub fn error(msg: &str) {
    log::error!("{}", msg.red());
}

/// Log a debug message
pub fn debug(msg: &str) {
    log::debug!("{}", msg);
}

/// Log a warning in yellow
pub fn warn(msg: &str) {
    log::warn!("{}", msg.yellow());
}

/// Log timing information
pub fn elapsed(what: &str, start: Instant) {
    let ms = start.elapsed().as_millis();
    log::info!("{} completed in {}ms", what.green(), ms);
}

/// Log the start of a step (nice for progress)
pub fn start_step(step: &str) -> Instant {
    log::info!("Starting {}", step.blue());
    Instant::now()
}
