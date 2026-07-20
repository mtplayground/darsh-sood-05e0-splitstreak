use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub app: AppConfig,
    pub auth: Option<AuthConfig>,
    pub database: DatabaseConfig,
    pub email: Option<EmailConfig>,
    pub legacy_jwt_secret: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub self_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthConfig {
    pub url: String,
    pub app_token: String,
    pub jwks_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailConfig {
    pub url: String,
    pub app_token: String,
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

        let app = AppConfig {
            self_url: read_optional_env("SELF_URL")?,
        };
        let auth = read_auth_config()?;
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
        let email = read_email_config()?;
        let legacy_jwt_secret = read_optional_env("JWT_SECRET")?;

        Ok(Self {
            host,
            port,
            app,
            auth,
            database: DatabaseConfig {
                url: database_url,
                max_connections: database_max_connections,
            },
            email,
            legacy_jwt_secret,
        })
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

fn read_auth_config() -> Result<Option<AuthConfig>, ConfigError> {
    let url = read_optional_env("MCTAI_AUTH_URL")?;
    let app_token = read_optional_env("MCTAI_AUTH_APP_TOKEN")?;
    let jwks_url = read_optional_env("MCTAI_AUTH_JWKS_URL")?;

    match (url, app_token, jwks_url) {
        (None, None, None) => Ok(None),
        (Some(url), Some(app_token), Some(jwks_url)) => Ok(Some(AuthConfig {
            url,
            app_token,
            jwks_url,
        })),
        _ => Err(ConfigError::PartialEnvGroup {
            group: "Ideavibes auth",
            keys: "MCTAI_AUTH_URL, MCTAI_AUTH_APP_TOKEN, MCTAI_AUTH_JWKS_URL",
        }),
    }
}

fn read_email_config() -> Result<Option<EmailConfig>, ConfigError> {
    let url = read_optional_env("MCTAI_EMAIL_URL")?;
    let app_token = read_optional_env("MCTAI_EMAIL_APP_TOKEN")?;

    match (url, app_token) {
        (None, None) => Ok(None),
        (Some(url), Some(app_token)) => Ok(Some(EmailConfig { url, app_token })),
        _ => Err(ConfigError::PartialEnvGroup {
            group: "Ideavibes email",
            keys: "MCTAI_EMAIL_URL, MCTAI_EMAIL_APP_TOKEN",
        }),
    }
}

fn read_optional_env(key: &'static str) -> Result<Option<String>, ConfigError> {
    match env::var(key) {
        Ok(value) if value.trim().is_empty() => Err(ConfigError::OptionalEnvEmpty { key }),
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::OptionalEnvNotUnicode { key }),
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
    OptionalEnvEmpty {
        key: &'static str,
    },
    OptionalEnvNotUnicode {
        key: &'static str,
    },
    PartialEnvGroup {
        group: &'static str,
        keys: &'static str,
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
            Self::OptionalEnvEmpty { key } => write!(formatter, "{key} must not be empty when set"),
            Self::OptionalEnvNotUnicode { key } => {
                write!(formatter, "{key} must be valid unicode when set")
            }
            Self::PartialEnvGroup { group, keys } => {
                write!(formatter, "{group} configuration must set all of: {keys}")
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
            | Self::OptionalEnvEmpty { .. }
            | Self::OptionalEnvNotUnicode { .. }
            | Self::PartialEnvGroup { .. }
            | Self::RequiredEnvEmpty { .. }
            | Self::RequiredEnvMissing { .. }
            | Self::RequiredEnvNotUnicode { .. } => None,
        }
    }
}
