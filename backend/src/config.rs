use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = match env::var("HOST") {
            Ok(value) => value.parse().map_err(|source| ConfigError::InvalidHost {
                value,
                source,
            })?,
            Err(env::VarError::NotPresent) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            Err(env::VarError::NotUnicode(_)) => return Err(ConfigError::HostNotUnicode),
        };

        let port = match env::var("PORT") {
            Ok(value) => value.parse().map_err(|source| ConfigError::InvalidPort {
                value,
                source,
            })?,
            Err(env::VarError::NotPresent) => 8080,
            Err(env::VarError::NotUnicode(_)) => return Err(ConfigError::PortNotUnicode),
        };

        let database_url = read_required_env("DATABASE_URL")?;
        let database_max_connections = match env::var("DATABASE_MAX_CONNECTIONS") {
            Ok(value) => {
                let parsed = value
                    .parse()
                    .map_err(|source| ConfigError::InvalidDatabaseMaxConnections {
                        value,
                        source,
                    })?;
                if parsed == 0 {
                    return Err(ConfigError::DatabaseMaxConnectionsMustBePositive);
                }
                parsed
            }
            Err(env::VarError::NotPresent) => 5,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::DatabaseMaxConnectionsNotUnicode);
            }
        };

        Ok(Self {
            host,
            port,
            database: DatabaseConfig {
                url: database_url,
                max_connections: database_max_connections,
            },
        })
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

fn read_required_env(key: &'static str) -> Result<String, ConfigError> {
    match env::var(key) {
        Ok(value) if value.trim().is_empty() => Err(ConfigError::RequiredEnvEmpty { key }),
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Err(ConfigError::RequiredEnvMissing { key }),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RequiredEnvNotUnicode { key }),
    }
}

#[derive(Debug)]
pub enum ConfigError {
    DatabaseMaxConnectionsMustBePositive,
    DatabaseMaxConnectionsNotUnicode,
    HostNotUnicode,
    InvalidDatabaseMaxConnections {
        value: String,
        source: std::num::ParseIntError,
    },
    InvalidHost {
        value: String,
        source: std::net::AddrParseError,
    },
    PortNotUnicode,
    InvalidPort {
        value: String,
        source: std::num::ParseIntError,
    },
    RequiredEnvEmpty {
        key: &'static str,
    },
    RequiredEnvMissing {
        key: &'static str,
    },
    RequiredEnvNotUnicode {
        key: &'static str,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseMaxConnectionsMustBePositive => {
                write!(formatter, "DATABASE_MAX_CONNECTIONS must be greater than zero")
            }
            Self::DatabaseMaxConnectionsNotUnicode => {
                write!(formatter, "DATABASE_MAX_CONNECTIONS must be valid unicode")
            }
            Self::HostNotUnicode => write!(formatter, "HOST must be valid unicode"),
            Self::InvalidDatabaseMaxConnections { value, source } => write!(
                formatter,
                "DATABASE_MAX_CONNECTIONS value '{value}' is invalid: {source}"
            ),
            Self::InvalidHost { value, source } => {
                write!(formatter, "HOST value '{value}' is invalid: {source}")
            }
            Self::PortNotUnicode => write!(formatter, "PORT must be valid unicode"),
            Self::InvalidPort { value, source } => {
                write!(formatter, "PORT value '{value}' is invalid: {source}")
            }
            Self::RequiredEnvEmpty { key } => write!(formatter, "{key} must not be empty"),
            Self::RequiredEnvMissing { key } => write!(formatter, "{key} is required"),
            Self::RequiredEnvNotUnicode { key } => write!(formatter, "{key} must be valid unicode"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidDatabaseMaxConnections { source, .. } => Some(source),
            Self::InvalidHost { source, .. } => Some(source),
            Self::InvalidPort { source, .. } => Some(source),
            Self::DatabaseMaxConnectionsMustBePositive
            | Self::DatabaseMaxConnectionsNotUnicode
            | Self::HostNotUnicode
            | Self::PortNotUnicode
            | Self::RequiredEnvEmpty { .. }
            | Self::RequiredEnvMissing { .. }
            | Self::RequiredEnvNotUnicode { .. } => None,
        }
    }
}
