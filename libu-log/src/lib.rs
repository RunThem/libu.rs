#![allow(non_snake_case)]
#![allow(unused)]

pub use log::*;

mod logger;

static LOGGER: logger::Logger = logger::Logger {
  level: Level::Trace,
};

pub fn init() {
  let _ = set_logger(&LOGGER).map(|()| set_max_level(LevelFilter::Trace));
}
