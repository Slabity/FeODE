use log::*;
use time::precise_time_ns;

use std::sync::Mutex;
use std::io::{Write, stdout};

// The message to be printed when the log is first opened.
static LOG_PRELUDE: &'static str = concat!("Fe2o3 Desktop Environment\n",
                                           "Version: ", env!("CARGO_PKG_VERSION"));

// Logger will allow the log to be written to any object that implements both the Write and Send
// trait, such as a File or a handle to Stdout or Stderr.
struct Logger {
    output: Mutex<Box<Write + Send>>,
    level: LogLevel,
    epoch: u64
}

// The Log trait allows Logger to be used with any of the macros from the log crate. See that crate
// for more information on the functions below.
impl Log for Logger {
    fn enabled(&self, meta_data: &LogMetadata) -> bool {
        meta_data.level() <= self.level
    }

    fn log(&self, record: &LogRecord) {
        let time = (precise_time_ns() - self.epoch) as f64 * 1e-9;
        let msg = format!("[{:.8}] ({})\t{}", time, record.level(), record.args());

        let ref mut guard = self.output.lock().unwrap();
        if let Some(err) = writeln!(*guard, "{}", msg).err() {

            // In a few rare cases, we may suddenly be unable to write to whatever output the
            // Logger is using, despite being able to before. This should not be considered a
            // critical error, so let's switch the log to use stderr.
            **guard = Box::new(stdout());
            if let Some(crit) = writeln!(*guard, "Error writing to log: {}", err).err() {

                // If we are somehow unable to write to stderr, then we should consider that a
                // critical error. Something has gone horribly wrong and we should not expect
                // normal behavior.
                panic!("Cannot write to stderr: {}", crit);
            }

            // We should make sure our original log message was printed out as well, in case the
            // message was important. But if we suddenly can't print to stderr, then we should
            // abort.
            if let Some(crit) = writeln!(*guard, "{}", msg).err() {
                panic!("Cannot write to stderr: {}", crit);
            }
        }
    }
}

pub fn init_logger(out: Box<Write + Send>, log_level: LogLevel) -> Result<(), SetLoggerError> {
    set_logger(|max_log_level| {
        max_log_level.set(log_level.to_log_level_filter());

        let logger = Box::new( Logger {
            output: Mutex::new(out),
            level: log_level,
            epoch: precise_time_ns()
        });

        {   // Block used to scope logger.output
            let ref mut guard = logger.output.lock().unwrap();
            if let Some(err) = writeln!(*guard, "{}", LOG_PRELUDE).err() {

                // For some reason we were passed a valid Box<Write + Send> object that we are
                // unable to write to. This may happen in a few rare cases, but should not be
                // considered a critical error. We should use stderr instead.
                **guard = Box::new(stdout());
                if let Some(crit) = writeln!(*guard, "Error writing to log: {}", err).err() {

                    // If we are somehow unable to write to stderr, then we should consider that a
                    // critical error. Something has gone horribly wrong and we should not expect
                    // normal behavior.
                    panic!("Cannot write to stderr: {}", crit);
                }
            }
        }

        logger
    })
}
