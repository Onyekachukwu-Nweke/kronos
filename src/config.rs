use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use crate::error::{Error, Result};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub databases: Databases,
    pub schedule: Option<Schedule>,
    pub storage: Storage,
}

#[derive(Deserialize, Debug)]
pub struct Databases {
    pub mysql: Option<DatabaseConfig>,
    pub postgres: Option<DatabaseConfig>,
    pub sqlite: Option<DatabaseConfig>,
    pub mongodb: Option<DatabaseConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub databases: Vec<String>, // List of database names to back up
}

#[derive(Deserialize, Debug)]
pub struct Schedule {
    pub cron: String, // Cron expression, e.g., "0 0 * * *" (daily at midnight)
}

#[derive(Deserialize, Debug)]
pub struct Storage {
    pub type_: String, // "local" or "s3"
    pub path: Option<String>, // Local storage path
    pub bucket: Option<String>, // S3 bucket
    pub region: Option<String>, // S3 region
    pub access_key: Option<String>, // S3 access key
    pub secret_key: Option<String>, // S3 secret key
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let mut file = File::open(path).map_err(|e| Error::Config(format!("Failed to open config file: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&contents)
            .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }
}