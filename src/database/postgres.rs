use crate::config::DatabaseConfig;
use crate::database::connection::{DatabaseConnection, DatabaseInfo, ConnectionStatus};
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use tokio::process::Command as AsyncCommand;

pub struct PostgreSQLDatabase<'a> {
    config: &'a DatabaseConfig,
}

impl<'a> PostgreSQLDatabase<'a> {
    pub fn new(config: &'a DatabaseConfig) -> Self {
        PostgreSQLDatabase { config }
    }

    fn get_connection_string(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/",
            self.config.user,
            self.config.password,
            self.config.host,
            self.config.port
        )
    }

    fn get_connection_args(&self) -> Vec<String> {
        vec![
            format!("--host={}", self.config.host),
            format!("--port={}", self.config.port),
            format!("--username={}", self.config.user),
        ]
    }

    async fn execute_psql_command(&self, database: &str, query: &str) -> Result<String> {
        let mut cmd = AsyncCommand::new("psql");
        cmd.args(&self.get_connection_args());
        cmd.args(&[
            format!("--dbname={}", database),
            "--no-password".to_string(),
            "--tuples-only".to_string(),
            "--no-align".to_string(),
            format!("--command={}", query),
        ]);
        
        // Set password via environment variable
        cmd.env("PGPASSWORD", &self.config.password);
        
        let output = cmd.output().await
            .map_err(|e| Error::Database(format!("Failed to execute psql command: {}", e)))?;
        
        if !output.status.success() {
            return Err(Error::Database(format!(
                "psql command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn execute_pg_dump(&self, database: &str, output_path: &Path) -> Result<()> {
        let mut cmd = AsyncCommand::new("pg_dump");
        cmd.args(&self.get_connection_args());
        cmd.args(&[
            format!("--dbname={}", database),
            "--no-password".to_string(),
            "--verbose".to_string(),
            "--clean".to_string(),
            "--create".to_string(),
            "--if-exists".to_string(),
            "--format=custom".to_string(),
        ]);
        
        // Set password via environment variable
        cmd.env("PGPASSWORD", &self.config.password);
        
        let output_file = output_path.join(format!("{}.dump", database));
        cmd.arg(format!("--file={}", output_file.to_string_lossy()));
        
        let output = cmd.output().await
            .map_err(|e| Error::Database(format!("Failed to execute pg_dump: {}", e)))?;
        
        if !output.status.success() {
            return Err(Error::Database(format!(
                "pg_dump failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        
        Ok(())
    }
}

#[async_trait]
impl<'a> DatabaseConnection for PostgreSQLDatabase<'a> {
    async fn test_connection(&self) -> Result<ConnectionStatus> {
        // Test connection with a simple query on the default postgres database
        match self.execute_psql_command("postgres", "SELECT 1;").await {
            Ok(_) => Ok(ConnectionStatus::Connected),
            Err(e) => Ok(ConnectionStatus::Error(e.to_string())),
        }
    }

    async fn get_database_info(&self) -> Result<Vec<DatabaseInfo>> {
        let mut info = Vec::new();
        
        for db_name in &self.config.databases {
            // Get database size
            let size_query = "SELECT pg_database_size(current_database());";
            let size_result = self.execute_psql_command(db_name, size_query).await?;
            let size = size_result.trim()
                .parse::<u64>()
                .ok();
            
            // Get PostgreSQL version
            let version_query = "SELECT version();";
            let version_result = self.execute_psql_command(db_name, version_query).await?;
            let version = version_result.lines()
                .next()
                .map(|s| s.trim().to_string());
            
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
            self.execute_pg_dump(db_name, backup_path).await?;
        }
        
        Ok(())
    }

    fn database_type(&self) -> &'static str {
        "postgres"
    }

    fn validate_config(&self, config: &DatabaseConfig) -> Result<()> {
        if config.host.is_empty() {
            return Err(Error::Config("PostgreSQL host cannot be empty".to_string()));
        }
        if config.user.is_empty() {
            return Err(Error::Config("PostgreSQL user cannot be empty".to_string()));
        }
        if config.databases.is_empty() {
            return Err(Error::Config("At least one database must be specified".to_string()));
        }
        Ok(())
    }

    async fn estimate_backup_size(&self) -> Result<u64> {
        let mut total_size = 0u64;
        
        for db_name in &self.config.databases {
            let size_query = "SELECT pg_database_size(current_database());";
            let size_result = self.execute_psql_command(db_name, size_query).await?;
            
            if let Ok(size) = size_result.trim().parse::<u64>() {
                total_size += size;
            }
        }
        
        // Add 15% overhead for dump format
        Ok((total_size as f64 * 1.15) as u64)
    }
}