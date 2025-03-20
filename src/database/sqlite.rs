use crate::config::DatabaseConfig;
use crate::error::{Error, Result};
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

    pub async fn backup(&self, backup_path: &Path) -> Result<()> {
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

            // Perform backup using SQLite Online Backup API
            let mut backup = Backup::new(&source_conn, &mut dest_conn)
                .map_err(|e| Error::Database(format!("Failed to initialize backup: {}", e)))?;

            backup.run_to_completion(-1, Duration::from_millis(1000), None) // -1 for full backup, 1000ms sleep between steps
                .map_err(|e| Error::Database(format!("Failed to execute backup: {}", e)))?;

            // Close connections explicitly
            source_conn.close()
                .map_err(|(_, e)| Error::Database(format!("Failed to close source connection: {}", e)))?;
            dest_conn.close()
                .map_err(|(_, e)| Error::Database(format!("Failed to close destination connection: {}", e)))?;
        }

        Ok(())
    }
}