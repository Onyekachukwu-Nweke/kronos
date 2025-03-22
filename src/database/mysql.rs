use std::io::{BufWriter, Write};
use crate::config::DatabaseConfig;
use crate::error::{Error, Result};
use sqlx::{Column, MySql, Pool, Row};
use std::path::Path;
use tokio::fs::File;
// use tokio::io::{AsyncWriteExt, BufWriter};

pub struct MySQLDatabase<'a> {
    config: &'a DatabaseConfig,
}

impl<'a> MySQLDatabase<'a> {
    pub fn new(config: &'a DatabaseConfig) -> Self {
        MySQLDatabase { config }
    }

    pub async fn backup(&self, backup_path: &Path) -> Result<()> {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&format!(
                "mysql://{}:{}@{}:{}/mysql",
                self.config.user, self.config.password, self.config.host, self.config.port
            ))
            .await
            .map_err(|e| Error::Database(format!("Failed to connect to MySQL: {}", e)))?;

        for db_name in &self.config.databases {
            let db_path = backup_path.join(db_name);
            tokio::fs::create_dir_all(&db_path)
                .await
                .map_err(Error::Io)?;

            let file = File::create(db_path.join("dump.sql"))
                .await
                .map_err(Error::Io)?;
            let mut writer = BufWriter::with_capacity(1024 * 1024, file); // 1MB buffer

            // Switch to the database
            sqlx::query(&format!("USE {}", db_name))
                .execute(&pool)
                .await
                .map_err(|e| Error::Database(format!("Failed to switch to database {}: {}", db_name, e)))?;

            // Dump schema
            let tables: Vec<(String,)> = sqlx::query_as("SHOW TABLES")
                .fetch_all(&pool)
                .await
                .map_err(|e| Error::Database(format!("Failed to fetch tables: {}", e)))?;

            for (table,) in tables {
                let create_stmt: (String,) = sqlx::query_as(&format!("SHOW CREATE TABLE {}", table))
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| Error::Database(format!("Failed to get schema for {}: {}", table, e)))?;
                writer.write_all(format!("{};\n", create_stmt.0).as_bytes())
                    .await
                    .map_err(Error::Io)?;
            }

            // Stream and dump data in batches
            for (table,) in &tables {
                let mut stream = sqlx::query(&format!("SELECT * FROM {}", table))
                    .fetch(&pool);
                let mut buffer = Vec::with_capacity(1000); // Buffer 1000 rows

                while let Some(row_result) = stream.next().await {
                    let row = row_result
                        .map_err(|e| Error::Database(format!("Failed to fetch row from {}: {}", table, e)))?;
                    let values: Vec<String> = row
                        .columns()
                        .iter()
                        .map(|col| {
                            row.try_get::<Option<String>, _>(col.name())
                                .unwrap_or(None)
                                .unwrap_or("NULL".to_string())
                        })
                        .collect();
                    buffer.push(format!("INSERT INTO {} VALUES ({});\n", table, values.join(", ")));

                    if buffer.len() >= 1000 {
                        writer.write_all(buffer.join("").as_bytes())
                            .await
                            .map_err(Error::Io)?;
                        buffer.clear();
                    }
                }

                // Write any remaining rows
                if !buffer.is_empty() {
                    writer.write_all(buffer.join("").as_bytes())
                        .await
                        .map_err(Error::Io)?;
                }
            }

            writer.flush().await.map_err(Error::Io)?;
        }

        pool.close().await;
        Ok(())
    }
}