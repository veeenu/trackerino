use simplelog::{
  CombinedLogger,
  SimpleLogger,
  TermLogger, TerminalMode,
  LevelFilter, ConfigBuilder, 
};

//
// Initialize logging
//
pub fn init() {
  let log_config = ConfigBuilder::new()
    .set_time_format_str("%Y-%m-%d %H:%M:%S")
    .build();
  CombinedLogger::init(vec![
    #[cfg(feature = "termcolor")]
    TermLogger::new(LevelFilter::Info, log_config, TerminalMode::Mixed),
    #[cfg(not(feature = "termcolor"))]
    SimpleLogger::new(LevelFilter::Info, log_config),
  ])
  .unwrap();
}
