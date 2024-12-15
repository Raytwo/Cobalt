use log::{Level, Metadata, Record};

pub struct KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            horizon_svc::output_debug_string(&record.args().to_string()).expect("could not convert the message to UTF8");
        }
    }

    fn flush(&self) {}
}