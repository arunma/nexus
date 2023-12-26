use anyhow::{anyhow, Context};
use axum_odbc::ODBCConnectionManager;
use dotenv::dotenv;
use secrecy::ExposeSecret;
use tera::Tera;
use tracing::error;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use nexus::config::{AppConfig, DbConfig};
use nexus::errors::ApiError;
use nexus::routes::AppController;
use nexus::service_register::ServiceRegister;
use nexus::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let config = AppConfig::get_configuration("config/customer_master.yaml")?; //TODO - get as args
    let api_host = config.api.host.to_string();
    let api_port = config.api.port;

    tracing_subscriber::registry()
        .with(EnvFilter::new(&config.api.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = create_db_manager(&config.db);
    let tera = create_tera(&config).unwrap();
    let app_state = AppState::new(config.clone());

    let service_register = ServiceRegister::new(config, pool, tera);

    AppController::serve(app_state, service_register)
        .await
        .context(format!("Unable to start server at host:port {}:{}", api_host, api_port))?;

    Ok(())
}

pub fn create_db_manager(db_cfg: &DbConfig) -> ODBCConnectionManager {
    let db_url = format!(
        "Driver={};server={};database={};schema={};warehouse={};role={};UID={};PWD={}",
        db_cfg.driver,
        db_cfg.hostname,
        db_cfg.database,
        db_cfg.schema,
        db_cfg.warehouse,
        db_cfg.role,
        db_cfg.username,
        db_cfg.password.expose_secret()
    );
    ODBCConnectionManager::new(db_url, 4)
}

fn create_tera(app_cfg: &AppConfig) -> Result<Tera, ApiError> {
    let mut tera = Tera::default();
    tera.autoescape_on(vec![]); //TODO - This could protect from sql injection.  Need to investigate further

    tera.add_raw_template("sql", &app_cfg.sql).map_err(|e| {
        error!("Error while rendering template: {}", e);
        anyhow!(format!("SqlTemplatingError {}", e.to_string()))
    })?;

    Ok(tera)
}
