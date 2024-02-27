use log::{
    set_logger, set_max_level, Level, LevelFilter, Log, Metadata, Record, SetLoggerError,
};
use riot_wrappers::println;

pub struct RiotLogger {
    level: LevelFilter,
}

// Static initialisations of the loggers
static VERBOSE_LOGGER: RiotLogger = RiotLogger { level: LevelFilter::Debug };
static INFO_LOGGER: RiotLogger = RiotLogger { level: LevelFilter::Info };
static ERROR_LOGGER: RiotLogger = RiotLogger { level: LevelFilter::Error };

impl RiotLogger {
    pub fn init(log_level: LevelFilter) -> Result<(), SetLoggerError> {
        set_max_level(log_level);
        let logger: &'static dyn Log = match log_level {
            LevelFilter::Off | LevelFilter::Error => &ERROR_LOGGER,
            LevelFilter::Warn | LevelFilter::Info => &INFO_LOGGER,
            LevelFilter::Debug | LevelFilter::Trace => &ERROR_LOGGER,
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
