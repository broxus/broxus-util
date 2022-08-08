use serde::Deserialize;

pub fn read_config<P, T>(path: P) -> Result<T, ConfigError>
where
    P: AsRef<std::path::Path>,
    for<'de> T: Deserialize<'de>,
{
    let data = std::fs::read_to_string(path)?;
    let re = regex::Regex::new(r"\$\{([a-zA-Z_][0-9a-zA-Z_]*)\}").unwrap();
    let result = re.replace_all(&data, |caps: &regex::Captures| {
        match std::env::var(&caps[1]) {
            Ok(value) => value,
            Err(_) => {
                eprintln!("WARN: Environment variable {} was not set", &caps[1]);
                String::default()
            }
        }
    });

    config::Config::builder()
        .add_source(config::File::from_str(
            result.as_ref(),
            config::FileFormat::Yaml,
        ))
        .build()
        .map_err(ConfigError::BuildError)?
        .try_deserialize()
        .map_err(ConfigError::ParseError)
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config")]
    UnableToRead(#[from] std::io::Error),
    #[error("failed to build config")]
    BuildError(#[source] config::ConfigError),
    #[error("failed to parse config")]
    ParseError(#[source] config::ConfigError),
}
