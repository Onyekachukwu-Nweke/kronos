use crate::config::DatabaseConfig;
use crate::database::connection::{DatabaseConnection, DatabaseInfo, ConnectionStatus};
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use tokio::process::Command as AsyncCommand;

pub struct MySQLDatabase<'a> {
    config: &'a DatabaseConfig,
}

impl<'a> MySQLDatabase<'a> {
    pub fn new(config: &'a DatabaseConfig) -> Self {
        MySQLDatabase { config }
    }

    fn get_connection_args(&self) -> Vec<String> {
        vec![
            format!("--host={}", self.config.host),
            format!("--port={}", self.config.port),
            format!("--user={}", self.config.user),
            format!("--password={}", self.config.password),
        ]
    }

    async fn execute_mysql_command(&self, args: &[String]) -> Result<String> {
        let mut cmd = AsyncCommand::new("mysql");
        cmd.args(&self.get_connection_args());
        cmd.args(args);
        
        let output = cmd.output().await
            .map_err(|e| Error::Database(format!("Failed to execute mysql command: {}", e)))?;
        
        if !output.status.success() {
            return Err(Error::Database(format!(
                "MySQL command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn execute_mysqldump(&self, database: &str, output_path: &Path) -> Result<()> {
        let mut cmd = AsyncCommand::new("mysqldump");
        cmd.args(&self.get_connection_args());
        cmd.args(&[
            "--single-transaction",
            "--routines",
            "--triggers",
            "--events",
            "--add-drop-database",
            "--create-options",
            database,
        ]);
        
        let output_file = output_path.join(format!("{}.sql", database));
        let output = cmd.output().await
            .map_err(|e| Error::Database(format!("Failed to execute mysqldump: {}", e)))?;
        
        if !output.status.success() {
            return Err(Error::Database(format!(
                "mysqldump failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        
        fs::write(&output_file, &output.stdout).await
            .map_err(|e| Error::Io(e))?;
        
        Ok(())
    }
}

#[async_trait]
impl<'a> DatabaseConnection for MySQLDatabase<'a> {
    async fn test_connection(&self) -> Result<ConnectionStatus> {
        match self.execute_mysql_command(&["--execute=SELECT 1".to_string()]).await {
            Ok(_) => Ok(ConnectionStatus::Connected),
            Err(e) => Ok(ConnectionStatus::Error(e.to_string())),
        }
    }

    async fn get_database_info(&self) -> Result<Vec<DatabaseInfo>> {
        let mut info = Vec::new();
        
        for db_name in &self.config.databases {
            let size_query = format!(
                "--execute=SELECT ROUND(SUM(data_length + index_length) / 1024 / 1024, 1) AS 'DB Size in MB' FROM information_schema.tables WHERE table_schema='{}'",
                db_name
            );
            
            let size_result = self.execute_mysql_command(&[size_query]).await?;
            let size = size_result.lines()
                .skip(1) // Skip header
                .next()
                .and_then(|line| line.parse::<f64>().ok())
                .map(|mb| (mb * 1024.0 * 1024.0) as u64);
            
            let version_query = "--execute=SELECT VERSION()".to_string();
            let version_result = self.execute_mysql_command(&[version_query]).await?;
            let version = version_result.lines()
                .skip(1)
                .next()
                .map(|s| s.to_string());
            
            info.push(DatabaseInfo {
                name: db_name.clone(),
                size,
                schema_version: version,
            });
        }
        
        Ok(info)
    }

    async fn backup(&self, backup_path: &Path) -> Result<()> {
        fs::create_dir_all(backup_path).await
            .map_err(Error::Io)?;
        
        for db_name in &self.config.databases {
            self.execute_mysqldump(db_name, backup_path).await?;
        }
        
        Ok(())
    }

    fn database_type(&self) -> &'static str {
        "mysql"
    }

    fn validate_config(&self, config: &DatabaseConfig) -> Result<()> {
        if config.host.is_empty() {
            return Err(Error::Config("MySQL host cannot be empty".to_string()));
        }
        if config.user.is_empty() {
            return Err(Error::Config("MySQL user cannot be empty".to_string()));
        }
        if config.databases.is_empty() {
            return Err(Error::Config("At least one database must be specified".to_string()));
        }
        Ok(())
    }

    async fn estimate_backup_size(&self) -> Result<u64> {
        let mut total_size = 0u64;
        
        for db_name in &self.config.databases {
            let size_query = format!(
                "--execute=SELECT COALESCE(SUM(data_length + index_length), 0) FROM information_schema.tables WHERE table_schema='{}'",
                db_name
            );
            
            let size_result = self.execute_mysql_command(&[size_query]).await?;
            if let Some(size_str) = size_result.lines().skip(1).next() {
                if let Ok(size) = size_str.parse::<u64>() {
                    total_size += size;
                }
            }
        }
        
        // Add 20% overhead for SQL dump format
        Ok((total_size as f64 * 1.2) as u64)
    }
}