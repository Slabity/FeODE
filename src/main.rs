#[macro_use]
extern crate log;
extern crate time;

mod logger;

use log::LogLevel;
use logger::init_logger;

fn main() {

    // TODO: Decide on a suitable place for the default log.
    if let Err(e) = init_logger(Box::new(std::io::stderr()), LogLevel::Trace) {

        // This error occurs if we already made a call to init_logger.
        panic!("Failed to initialize the log: {}", e);
    }
}
