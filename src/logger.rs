use log::{LevelFilter, Log, Metadata, Record};

// To import write_serial!
use crate::*;

static LOGGER: SerialLogger = SerialLogger;

struct SerialLogger;

impl Log for SerialLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        write_serial!(
            "[{level:<5}] [{target}] {args}\n",
            level = record.level(),
            target = record.target(),
            args = record.args()
        );
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .expect("Initialize logger");
}
