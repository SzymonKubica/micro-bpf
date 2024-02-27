use log::{set_logger, set_max_level, Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use riot_wrappers::println;

/* Because we are running under no_std, we cannot use the set_boxed_logger
function to tell the log crate which logger to use. Because of this, we
need to use the set_logger function which requires that the logger passed
into it is 'static. */

static OFF_LOGGER: RiotLogger = RiotLogger::new(LevelFilter::Off);
static ERROR_LOGGER: RiotLogger = RiotLogger::new(LevelFilter::Error);
static WARN_LOGGER: RiotLogger = RiotLogger::new(LevelFilter::Warn);
static DEBUG_LOGGER: RiotLogger = RiotLogger::new(LevelFilter::Debug);
static INFO_LOGGER: RiotLogger = RiotLogger::new(LevelFilter::Info);
static TRACE_LOGGER: RiotLogger = RiotLogger::new(LevelFilter::Trace);

/// Simle logger that logs into the RIOT shell console output by using the
/// println! macro.
pub struct RiotLogger {
    level: LevelFilter,
}

impl RiotLogger {
    pub const fn new(level: LevelFilter) -> Self {
        RiotLogger { level }
    }
    pub fn init(log_level: LevelFilter) -> Result<(), SetLoggerError> {
        set_max_level(log_level);
        let logger: &'static dyn Log = match log_level {
            LevelFilter::Off => &OFF_LOGGER,
            LevelFilter::Error => &ERROR_LOGGER,
            LevelFilter::Warn => &WARN_LOGGER,
            LevelFilter::Info => &INFO_LOGGER,
            LevelFilter::Debug => &DEBUG_LOGGER,
            LevelFilter::Trace => &TRACE_LOGGER,
        };
        set_logger(logger)
    }
}
impl Log for RiotLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            println!(
                "{}:{} -- {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}


pub fn log_thread_spawned(thread: &CountedThread, thread_name: &str) {
    debug!(
        "{} thread spawned as {:?} ({:?}), status {:?}",
        thread_name,
        thread.pid(),
        thread.pid().get_name(),
        thread.status()
    );
}
