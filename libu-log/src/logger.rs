use colored::Colorize;
use log::Level;

pub struct Logger {
  pub(crate) level: Level,
}

impl log::Log for Logger {
  fn enabled(&self, metadata: &log::Metadata) -> bool {
    metadata.level() <= self.level
  }

  fn log(&self, record: &log::Record) {
    if self.enabled(record.metadata()) {
      let time = chrono::Local::now();
      let args = record.args();
      let line = record.line().unwrap_or(0);
      let file = record.file().unwrap_or("???");

      let prefix = match record.level() {
        Level::Error => format!("[{} ERR]", time.format("%H:%M:%S")).red(),
        Level::Warn => format!("[{} WAR]", time.format("%H:%M:%S")).yellow(),
        Level::Info => format!("[{} INF]", time.format("%H:%M:%S")).green(),
        Level::Debug => format!("[{} DBG]", time.format("%H:%M:%S")).cyan(),
        Level::Trace => format!("[{} TRC]", time.format("%H:%M:%S")).purple(),
      };

      println!("{prefix}: {}. <{}:{}>", args, file, line);
    }
  }

  fn flush(&self) {}
}
