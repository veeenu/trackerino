use simplelog::{
  CombinedLogger, ConfigBuilder, LevelFilter, SimpleLogger, TermLogger, TerminalMode,
};

//
// Initialize logging
//
pub fn init() {
  let level_filter = match std::env::var("RUST_LOG").ok().as_ref().map(String::as_str) {
    Some("off") => LevelFilter::Off,
    Some("error") => LevelFilter::Error,
    Some("warn") => LevelFilter::Warn,
    Some("info") => LevelFilter::Info,
    Some("debug") => LevelFilter::Debug,
    Some("trace") => LevelFilter::Trace,
    _ => LevelFilter::Info,
  };

  let log_config = ConfigBuilder::new()
    .set_time_format_str("%Y-%m-%d %H:%M:%S")
    .build();
  CombinedLogger::init(vec![
    #[cfg(feature = "termcolor")]
    TermLogger::new(level_filter, log_config, TerminalMode::Mixed),
    #[cfg(not(feature = "termcolor"))]
    SimpleLogger::new(level_filter, log_config),
  ])
  .unwrap();
}
