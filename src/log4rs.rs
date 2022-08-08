pub fn init_logger(initial_value: &serde_yaml::Value) -> Result<log4rs::Handle, LoggerError> {
    let handle = log4rs::config::init_config(parse_logger_config(initial_value.clone())?)?;
    Ok(handle)
}

pub fn parse_logger_config(value: serde_yaml::Value) -> Result<log4rs::Config, LoggerError> {
    let config = serde_yaml::from_value::<log4rs::config::RawConfig>(value)?;

    let (appenders, errors) = config.appenders_lossy(&log4rs::config::Deserializers::default());
    if !errors.is_empty() {
        return Err(LoggerError::InvalidAppenders(format!("{errors:#?}")));
    }

    log4rs::Config::builder()
        .appenders(appenders)
        .loggers(config.loggers())
        .build(config.root())
        .map_err(LoggerError::BuildError)
}

#[derive(thiserror::Error, Debug)]
pub enum LoggerError {
    #[error("bad config")]
    InvalidConfig(#[from] serde_yaml::Error),
    #[error("invalid appenders: {0}")]
    InvalidAppenders(String),
    #[error("failed to build logger")]
    BuildError(#[from] log4rs::config::runtime::ConfigErrors),
    #[error("failed to set logger")]
    InitializationError(#[from] log::SetLoggerError),
}
