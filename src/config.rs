use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub struct Config {
    #[clap(long, env)]
    pub api_host: String,

    #[clap(long, env)]
    pub api_port: u16,

    #[clap(long, env)]
    pub rust_log: String,

    #[clap(long, env)]
    pub database_url: String,

    #[clap(long, env)]
    pub redis_url: String,

    #[clap(long, env)]
    pub client_origin: String,

    #[clap(long, env)]
    pub access_token_secret: String,

    #[clap(long, env)]
    pub access_token_expires_in: String,

    #[clap(long, env)]
    pub access_token_max_age: i64,

    #[clap(long, env)]
    pub refresh_token_secret: String,

    #[clap(long, env)]
    pub refresh_token_expires_in: String,

    #[clap(long, env)]
    pub refresh_token_max_age: i64,

    #[clap(long, env)]
    pub password_salt: String,
}