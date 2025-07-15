use crate::config::DatabaseConfig;
use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

/// Database connection metadata
#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    pub name: String,
    pub size: Option<u64>, // Size in bytes, if available
    pub schema_version: Option<String>,
}

/// Connection health status
#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error(String),
}

/// Trait for database connections that supports backup operations
#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    /// Test the connection to the database
    async fn test_connection(&self) -> Result<ConnectionStatus>;
    
    /// Get information about available databases
    async fn get_database_info(&self) -> Result<Vec<DatabaseInfo>>;
    
    /// Perform backup of specified databases to the given path
    async fn backup(&self, backup_path: &Path) -> Result<()>;
    
    /// Get the database type name (e.g., "mysql", "postgres", "sqlite", "mongodb")
    fn database_type(&self) -> &'static str;
    
    /// Validate configuration for this database type
    fn validate_config(&self, config: &DatabaseConfig) -> Result<()>;
    
    /// Get estimated backup size for planning purposes
    async fn estimate_backup_size(&self) -> Result<u64>;
}

/// Factory for creating database connections
pub struct DatabaseConnectionFactory;

impl DatabaseConnectionFactory {
    /// Create a database connection based on type
    pub fn create_connection<'a>(
        db_type: &str,
        config: &'a DatabaseConfig,
    ) -> Result<Box<dyn DatabaseConnection + 'a>> {
        match db_type {
            "mysql" => Ok(Box::new(super::mysql::MySQLDatabase::new(config))),
            "postgres" => Ok(Box::new(super::postgres::PostgreSQLDatabase::new(config))),
            "sqlite" => Ok(Box::new(super::sqlite::SQLiteDatabase::new(config))),
            "mongodb" => Ok(Box::new(super::mongodb::MongoDatabase::new(config))),
            _ => Err(crate::error::Error::Database(format!(
                "Unsupported database type: {}",
                db_type
            ))),
        }
    }

    /// Get list of supported database types
    pub fn supported_types() -> Vec<&'static str> {
        vec!["mysql", "postgres", "sqlite", "mongodb"]
    }
}