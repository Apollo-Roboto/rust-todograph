use colored::Colorize;
use log::{Level, Log, Metadata, Record};

pub struct SimpleLogger;

pub static LOGGER: SimpleLogger = SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level_text = match record.level() {
            Level::Error => "ERROR".bright_red(),
            Level::Warn => "WARNING".yellow(),
            Level::Info => "INFO".green(),
            Level::Debug => "DEBUG".cyan(),
            Level::Trace => "TRACE".magenta(),
        };

        let module = record.target().bright_black();

        let now = chrono::Local::now();
        let now_text = now.format("%Y-%m-%d %H:%M:%S").to_string().bright_black();

        for (i, line) in record.args().to_string().lines().enumerate() {
            match i {
                0 => println!("{:<19} {:<7} {} {}", now_text, level_text, module, line),
                _ => println!("{:<19} {:<7} {} {}", "", "", module, line),
            }
        }
    }

    fn flush(&self) {}
}
