use crate::config::DatabaseConfig;
use crate::database::connection::{DatabaseConnection, DatabaseInfo, ConnectionStatus};
use crate::error::{Error, Result};
use async_trait::async_trait;
use rusqlite::{Connection, OpenFlags, backup::Backup};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;

pub struct SQLiteDatabase<'a> {
    config: &'a DatabaseConfig,
}

impl<'a> SQLiteDatabase<'a> {
    pub fn new(config: &'a DatabaseConfig) -> Self {
        SQLiteDatabase { config }
    }

    async fn backup_database(&self, backup_path: &Path) -> Result<()> {
        for db_name in &self.config.databases {
            // Construct source database path
            let source_path = PathBuf::from(&self.config.host).join(db_name);
            if !source_path.exists() {
                return Err(Error::Database(format!("Database file not found: {:?}", source_path)));
            }

            // Construct destination backup path
            let backup_file_name = format!("{}.bak", db_name);
            let dest_path = backup_path.join(&backup_file_name);
            fs::create_dir_all(backup_path)
                .await
                .map_err(Error::Io)?;

            // Open source database connection
            let source_conn = Connection::open_with_flags(
                &source_path,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
                .map_err(|e| Error::Database(format!("Failed to open source SQLite DB: {}", e)))?;

            // Open or create destination database connection
            let mut dest_conn = Connection::open(&dest_path)
                .map_err(|e| Error::Database(format!("Failed to open destination SQLite DB: {}", e)))?;

            // Perform backup within a scope to drop `backup` before closing connections
            {
                let backup = Backup::new(&source_conn, &mut dest_conn)
                    .map_err(|e| Error::Database(format!("Failed to initialize backup: {}", e)))?;

                backup.run_to_completion(10, Duration::from_millis(1000), None) // -1 for full backup, 1000ms sleep between steps
                    .map_err(|e| Error::Database(format!("Failed to execute backup: {}", e)))?;
            } // `backup` is dropped here, ending the borrow

            // Now safe to close connections
            source_conn.close()
                .map_err(|(_, e)| Error::Database(format!("Failed to close source connection: {}", e)))?;
            dest_conn.close()
                .map_err(|(_, e)| Error::Database(format!("Failed to close destination connection: {}", e)))?;
        }

        Ok(())
    }

    fn get_database_file_size(&self, db_path: &Path) -> Result<u64> {
        let metadata = std::fs::metadata(db_path)
            .map_err(|e| Error::Database(format!("Failed to get database file size: {}", e)))?;
        Ok(metadata.len())
    }

    fn test_database_connection(&self, db_path: &Path) -> Result<()> {
        let _conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| Error::Database(format!("Failed to open SQLite database: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl<'a> DatabaseConnection for SQLiteDatabase<'a> {
    async fn test_connection(&self) -> Result<ConnectionStatus> {
        for db_name in &self.config.databases {
            let db_path = PathBuf::from(&self.config.host).join(db_name);
            if let Err(e) = self.test_database_connection(&db_path) {
                return Ok(ConnectionStatus::Error(e.to_string()));
            }
        }
        Ok(ConnectionStatus::Connected)
    }

    async fn get_database_info(&self) -> Result<Vec<DatabaseInfo>> {
        let mut info = Vec::new();
        
        for db_name in &self.config.databases {
            let db_path = PathBuf::from(&self.config.host).join(db_name);
            
            let size = if db_path.exists() {
                self.get_database_file_size(&db_path).ok()
            } else {
                None
            };
            
            // Get SQLite version
            let version = if db_path.exists() {
                match Connection::open_with_flags(
                    &db_path,
                    OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
                ) {
                    Ok(conn) => {
                        let version: std::result::Result<String, rusqlite::Error> = conn.query_row(
                            "SELECT sqlite_version()",
                            [],
                            |row| row.get(0)
                        );
                        version.ok()
                    }
                    Err(_) => None,
                }
            } else {
                None
            };
            
            info.push(DatabaseInfo {
                name: db_name.clone(),
                size,
                schema_version: version,
            });
        }
        
        Ok(info)
    }

    async fn backup(&self, backup_path: &Path) -> Result<()> {
        self.backup_database(backup_path).await
    }

    fn database_type(&self) -> &'static str {
        "sqlite"
    }

    fn validate_config(&self, config: &DatabaseConfig) -> Result<()> {
        if config.host.is_empty() {
            return Err(Error::Config("SQLite host (directory path) cannot be empty".to_string()));
        }
        if config.databases.is_empty() {
            return Err(Error::Config("At least one database file must be specified".to_string()));
        }
        
        // Check if the directory exists
        let host_path = Path::new(&config.host);
        if !host_path.exists() {
            return Err(Error::Config(format!("SQLite host directory does not exist: {}", config.host)));
        }
        
        // Check if specified database files exist
        for db_name in &config.databases {
            let db_path = host_path.join(db_name);
            if !db_path.exists() {
                return Err(Error::Config(format!("SQLite database file does not exist: {:?}", db_path)));
            }
        }
        
        Ok(())
    }

    async fn estimate_backup_size(&self) -> Result<u64> {
        let mut total_size = 0u64;
        
        for db_name in &self.config.databases {
            let db_path = PathBuf::from(&self.config.host).join(db_name);
            if let Ok(size) = self.get_database_file_size(&db_path) {
                total_size += size;
            }
        }
        
        // SQLite backup is nearly the same size as the original
        Ok(total_size)
    }
}