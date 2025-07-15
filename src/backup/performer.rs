use crate::config::Config;
use crate::database::connection::{DatabaseConnectionFactory, DatabaseConnection};
use crate::error::{Error, Result};
use std::path::Path;
use log::info;

pub struct BackupPerformer<'a> {
    config: &'a Config,
    backup_path: &'a Path,
}

impl<'a> BackupPerformer<'a> {
    pub fn new(config: &'a Config, backup_path: &'a Path) -> Self {
        BackupPerformer { config, backup_path }
    }

    pub async fn execute(&mut self) -> Result<()> {
        let mut backup_completed = false;

        // Handle SQLite databases
        if let Some(sqlite_config) = &self.config.databases.sqlite {
            info!("Starting SQLite backup");
            let db = DatabaseConnectionFactory::create_connection("sqlite", sqlite_config)?;
            self.perform_backup(&*db, "sqlite").await?;
            backup_completed = true;
        }

        // Handle MySQL databases
        if let Some(mysql_config) = &self.config.databases.mysql {
            info!("Starting MySQL backup");
            let db = DatabaseConnectionFactory::create_connection("mysql", mysql_config)?;
            self.perform_backup(&*db, "mysql").await?;
            backup_completed = true;
        }

        // Handle PostgreSQL databases
        if let Some(postgres_config) = &self.config.databases.postgres {
            info!("Starting PostgreSQL backup");
            let db = DatabaseConnectionFactory::create_connection("postgres", postgres_config)?;
            self.perform_backup(&*db, "postgres").await?;
            backup_completed = true;
        }

        // Handle MongoDB databases
        if let Some(mongodb_config) = &self.config.databases.mongodb {
            info!("Starting MongoDB backup");
            let db = DatabaseConnectionFactory::create_connection("mongodb", mongodb_config)?;
            self.perform_backup(&*db, "mongodb").await?;
            backup_completed = true;
        }

        if !backup_completed {
            return Err(Error::Config("No database configurations found".to_string()));
        }

        Ok(())
    }

    async fn perform_backup(&self, db: &dyn DatabaseConnection, db_type: &str) -> Result<()> {
        // Test connection first
        let status = db.test_connection().await?;
        match status {
            crate::database::connection::ConnectionStatus::Connected => {
                info!("Successfully connected to {} database", db_type);
            }
            crate::database::connection::ConnectionStatus::Error(e) => {
                return Err(Error::Database(format!("Failed to connect to {} database: {}", db_type, e)));
            }
            crate::database::connection::ConnectionStatus::Disconnected => {
                return Err(Error::Database(format!("{} database is disconnected", db_type)));
            }
        }

        // Get database info
        let db_info = db.get_database_info().await?;
        info!("Found {} databases for backup:", db_info.len());
        for info in &db_info {
            let size_str = match info.size {
                Some(size) => format!("{} bytes", size),
                None => "unknown size".to_string(),
            };
            info!("  - {} ({})", info.name, size_str);
        }

        // Estimate backup size
        let estimated_size = db.estimate_backup_size().await?;
        info!("Estimated backup size: {} bytes", estimated_size);

        // Perform the backup
        info!("Starting backup for {} databases", db_type);
        db.backup(self.backup_path).await?;
        info!("Backup completed successfully for {} databases", db_type);

        Ok(())
    }
}