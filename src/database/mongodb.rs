use crate::config::DatabaseConfig;
use crate::database::connection::{DatabaseConnection, DatabaseInfo, ConnectionStatus};
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use tokio::process::Command as AsyncCommand;
use serde_json::Value;

pub struct MongoDatabase<'a> {
    config: &'a DatabaseConfig,
}

impl<'a> MongoDatabase<'a> {
    pub fn new(config: &'a DatabaseConfig) -> Self {
        MongoDatabase { config }
    }

    fn get_connection_string(&self) -> String {
        format!(
            "mongodb://{}:{}@{}:{}/",
            self.config.user,
            self.config.password,
            self.config.host,
            self.config.port
        )
    }

    fn get_connection_args(&self) -> Vec<String> {
        vec![
            format!("--host={}:{}", self.config.host, self.config.port),
            format!("--username={}", self.config.user),
            format!("--password={}", self.config.password),
            "--authenticationDatabase=admin".to_string(),
        ]
    }

    async fn execute_mongo_command(&self, database: &str, command: &str) -> Result<String> {
        let mut cmd = AsyncCommand::new("mongo");
        cmd.args(&self.get_connection_args());
        cmd.args(&[
            database,
            "--quiet",
            "--eval",
            command,
        ]);
        
        let output = cmd.output().await
            .map_err(|e| Error::Database(format!("Failed to execute mongo command: {}", e)))?;
        
        if !output.status.success() {
            return Err(Error::Database(format!(
                "mongo command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn execute_mongodump(&self, database: &str, output_path: &Path) -> Result<()> {
        let mut cmd = AsyncCommand::new("mongodump");
        cmd.args(&self.get_connection_args());
        cmd.args(&[
            format!("--db={}", database),
            format!("--out={}", output_path.to_string_lossy()),
            "--gzip".to_string(),
        ]);
        
        let output = cmd.output().await
            .map_err(|e| Error::Database(format!("Failed to execute mongodump: {}", e)))?;
        
        if !output.status.success() {
            return Err(Error::Database(format!(
                "mongodump failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        
        Ok(())
    }

    async fn get_database_stats(&self, database: &str) -> Result<DatabaseInfo> {
        let stats_command = "JSON.stringify(db.stats())";
        let stats_result = self.execute_mongo_command(database, stats_command).await?;
        
        let size = if let Ok(stats) = serde_json::from_str::<Value>(&stats_result) {
            stats["dataSize"].as_u64()
        } else {
            None
        };
        
        let version_command = "JSON.stringify(db.version())";
        let version_result = self.execute_mongo_command(database, version_command).await?;
        let version = version_result.trim().trim_matches('"').to_string();
        
        Ok(DatabaseInfo {
            name: database.to_string(),
            size,
            schema_version: Some(version),
        })
    }
}

#[async_trait]
impl<'a> DatabaseConnection for MongoDatabase<'a> {
    async fn test_connection(&self) -> Result<ConnectionStatus> {
        match self.execute_mongo_command("admin", "db.runCommand('ping')").await {
            Ok(_) => Ok(ConnectionStatus::Connected),
            Err(e) => Ok(ConnectionStatus::Error(e.to_string())),
        }
    }

    async fn get_database_info(&self) -> Result<Vec<DatabaseInfo>> {
        let mut info = Vec::new();
        
        for db_name in &self.config.databases {
            match self.get_database_stats(db_name).await {
                Ok(db_info) => info.push(db_info),
                Err(e) => {
                    // If we can't get stats, still include the database with minimal info
                    info.push(DatabaseInfo {
                        name: db_name.clone(),
                        size: None,
                        schema_version: None,
                    });
                    log::warn!("Failed to get stats for database {}: {}", db_name, e);
                }
            }
        }
        
        Ok(info)
    }

    async fn backup(&self, backup_path: &Path) -> Result<()> {
        fs::create_dir_all(backup_path).await
            .map_err(Error::Io)?;
        
        for db_name in &self.config.databases {
            self.execute_mongodump(db_name, backup_path).await?;
        }
        
        Ok(())
    }

    fn database_type(&self) -> &'static str {
        "mongodb"
    }

    fn validate_config(&self, config: &DatabaseConfig) -> Result<()> {
        if config.host.is_empty() {
            return Err(Error::Config("MongoDB host cannot be empty".to_string()));
        }
        if config.user.is_empty() {
            return Err(Error::Config("MongoDB user cannot be empty".to_string()));
        }
        if config.password.is_empty() {
            return Err(Error::Config("MongoDB password cannot be empty".to_string()));
        }
        if config.databases.is_empty() {
            return Err(Error::Config("At least one database must be specified".to_string()));
        }
        Ok(())
    }

    async fn estimate_backup_size(&self) -> Result<u64> {
        let mut total_size = 0u64;
        
        for db_name in &self.config.databases {
            let stats_command = "JSON.stringify(db.stats())";
            match self.execute_mongo_command(db_name, stats_command).await {
                Ok(stats_result) => {
                    if let Ok(stats) = serde_json::from_str::<Value>(&stats_result) {
                        if let Some(size) = stats["dataSize"].as_u64() {
                            total_size += size;
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to get stats for database {}: {}", db_name, e);
                }
            }
        }
        
        // Add 25% overhead for BSON format and compression
        Ok((total_size as f64 * 1.25) as u64)
    }
}