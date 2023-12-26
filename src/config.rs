use std::collections::HashMap;
use std::net::{AddrParseError, SocketAddr};

use anyhow::Result as AResult;
use config::{Config, Environment, FileFormat};
use dotenv::dotenv;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::errors::ApiError;
use crate::errors::ApiError::ApplicationStartup;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub name: String,
    pub request: HashMap<String, String>,
    pub response: HashMap<String, String>,
    pub sql: String,
    pub db: DbConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DbConfig {
    pub driver: String,
    pub hostname: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: Secret<String>,
    pub database: String,
    pub warehouse: String,
    pub schema: String,
    pub role: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    pub port: u16,
    pub host: String,
    pub endpoint: String,
    pub rust_log: String,
    #[serde(skip_serializing)]
    pub access_token_secret: String,
    pub access_token_expires_in: String,
    pub access_token_max_age: usize,
    #[serde(skip_serializing)]
    pub password_salt: String,
}

impl AppConfig {
    pub fn get_configuration(file_path: &str) -> AResult<AppConfig> {
        dotenv().ok(); //Load .env file. For Prod, create a function and load the injected secrets as environment variables

        let config = Config::builder()
            .add_source(config::File::new(file_path, FileFormat::Yaml))
            .add_source(
                Environment::with_prefix("NEXUS")
                    .try_parsing(true)
                    .prefix_separator("__")
                    .separator("_"),
            )
            .build()?;

        let app_cfg: AppConfig = config.try_deserialize()?;
        info!("Loaded configuration: {:?}", app_cfg);

        Ok(app_cfg)
    }
    pub fn get_socket_address(app_cfg: &AppConfig) -> Result<SocketAddr, ApiError> {
        let address = format!("{}:{}", app_cfg.api.host, app_cfg.api.port);
        address.parse::<SocketAddr>().map_err(|e: AddrParseError| {
            eprintln!("Error while parsing socket address: {}", e);
            ApplicationStartup(e.to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use secrecy::ExposeSecret;

    use crate::config::AppConfig;

    #[test]
    fn test_file_and_dotenv_load() {
        let app_cfg = AppConfig::get_configuration("tests/test_customer_master.yaml").unwrap();
        assert_eq!(app_cfg.name, "customer_master");
        assert_eq!(app_cfg.api.endpoint, "/api/nexus/customer_master");
        assert_eq!(app_cfg.api.port, 8080);
        assert_ne!(app_cfg.db.username, "PLACEHOLDER_USERNAME");
        assert_ne!(app_cfg.db.password.expose_secret(), "PLACEHOLDER_PASSWORD");
    }
}
