use std::{env, error::Error, fmt, str::FromStr};

#[derive(Debug)]
pub struct ConfigError {
    message: String,
}

impl ConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Configuration error: {}", self.message)
    }
}

impl Error for ConfigError {}

#[derive(Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Clone)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

fn parse_env_value<T>(name: &str, value: String) -> Result<T, ConfigError>
where
    T: FromStr,
{
    if value.trim().is_empty() {
        return Err(ConfigError::new(format!("{name} cannot be empty")));
    }

    value
        .parse::<T>()
        .map_err(|_| ConfigError::new(format!("{name} has an invalid value")))
}

fn parse_env_with_default<T>(
    name: &str,
    value: Option<String>,
    default: T,
) -> Result<T, ConfigError>
where
    T: FromStr,
{
    match value {
        Some(value) => parse_env_value(name, value),
        None => Ok(default),
    }
}

fn get_env<T>(name: &str) -> Result<T, ConfigError>
where
    T: FromStr,
{
    let value = env::var(name).map_err(|error| match error {
        env::VarError::NotPresent => {
            ConfigError::new(format!("{name} is required but was not set"))
        }
        env::VarError::NotUnicode(_) => ConfigError::new(format!("{name} is not valid Unicode")),
    })?;

    parse_env_value(name, value)
}

fn get_env_with_default<T>(name: &str, default: T) -> Result<T, ConfigError>
where
    T: FromStr,
{
    let value = match env::var(name) {
        Ok(value) => Some(value),
        Err(env::VarError::NotPresent) => None,
        Err(env::VarError::NotUnicode(_)) => {
            return Err(ConfigError::new(format!("{name} is not valid Unicode")));
        }
    };

    parse_env_with_default(name, value, default)
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port: u16 = get_env_with_default("APP_PORT", 3000)?;
        let database_url: String = get_env("DATABASE_URL")?;

        Ok(Self {
            server: ServerConfig { port },
            database: DatabaseConfig { url: database_url },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_port() {
        let port: u16 = parse_env_value("APP_PORT", "5000".to_string()).unwrap();

        assert_eq!(port, 5000);
    }

    #[test]
    fn rejects_invalid_port() {
        let result = parse_env_value::<u16>("APP_PORT", "hello".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn rejects_port_out_of_range() {
        let result = parse_env_value::<u16>("APP_PORT", "70000".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn rejects_empty_value() {
        let result = parse_env_value::<u16>("APP_PORT", String::new());

        assert!(result.is_err());
    }

    #[test]
    fn uses_default_when_value_is_missing() {
        let port: u16 = parse_env_with_default("APP_PORT", None, 4000).unwrap();

        assert_eq!(port, 4000);
    }

    #[test]
    fn parses_value_when_present() {
        let port: u16 = parse_env_with_default("APP_PORT", Some("5000".to_string()), 4000).unwrap();

        assert_eq!(port, 5000);
    }

    #[test]
    fn rejects_invalid_value_instead_of_using_default() {
        let result = parse_env_with_default::<u16>("APP_PORT", Some("hello".to_string()), 4000);

        assert!(result.is_err());
    }
}
