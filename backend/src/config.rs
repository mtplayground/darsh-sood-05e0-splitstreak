use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
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

        Ok(Self { host, port })
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    HostNotUnicode,
    InvalidHost {
        value: String,
        source: std::net::AddrParseError,
    },
    PortNotUnicode,
    InvalidPort {
        value: String,
        source: std::num::ParseIntError,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HostNotUnicode => write!(formatter, "HOST must be valid unicode"),
            Self::InvalidHost { value, source } => {
                write!(formatter, "HOST value '{value}' is invalid: {source}")
            }
            Self::PortNotUnicode => write!(formatter, "PORT must be valid unicode"),
            Self::InvalidPort { value, source } => {
                write!(formatter, "PORT value '{value}' is invalid: {source}")
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidHost { source, .. } => Some(source),
            Self::InvalidPort { source, .. } => Some(source),
            Self::HostNotUnicode | Self::PortNotUnicode => None,
        }
    }
}
