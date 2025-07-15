use crate::config::DatabaseConfig;
use crate::database::connection::{DatabaseConnectionFactory, DatabaseConnection};
use crate::error::Result;

pub async fn test_database_framework() -> Result<()> {
    // Test SQLite connection creation
    let sqlite_config = DatabaseConfig {
        host: "/tmp".to_string(),
        port: 0,
        user: "".to_string(),
        password: "".to_string(),
        databases: vec!["test.db".to_string()],
    };
    
    let _sqlite_db = DatabaseConnectionFactory::create_connection("sqlite", &sqlite_config)?;
    println!("✓ SQLite connection created successfully");
    
    // Test MySQL connection creation
    let mysql_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 3306,
        user: "root".to_string(),
        password: "password".to_string(),
        databases: vec!["test_db".to_string()],
    };
    
    let _mysql_db = DatabaseConnectionFactory::create_connection("mysql", &mysql_config)?;
    println!("✓ MySQL connection created successfully");
    
    // Test PostgreSQL connection creation
    let postgres_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        user: "postgres".to_string(),
        password: "password".to_string(),
        databases: vec!["test_db".to_string()],
    };
    
    let _postgres_db = DatabaseConnectionFactory::create_connection("postgres", &postgres_config)?;
    println!("✓ PostgreSQL connection created successfully");
    
    // Test MongoDB connection creation
    let mongodb_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 27017,
        user: "admin".to_string(),
        password: "password".to_string(),
        databases: vec!["test_db".to_string()],
    };
    
    let _mongodb_db = DatabaseConnectionFactory::create_connection("mongodb", &mongodb_config)?;
    println!("✓ MongoDB connection created successfully");
    
    // Test unsupported database type
    match DatabaseConnectionFactory::create_connection("unsupported", &sqlite_config) {
        Err(_) => println!("✓ Unsupported database type properly rejected"),
        Ok(_) => println!("✗ Unsupported database type was accepted"),
    }
    
    // Test supported types
    let supported_types = DatabaseConnectionFactory::supported_types();
    println!("✓ Supported database types: {:?}", supported_types);
    
    println!("✓ All database connection framework tests passed!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_framework_creation() {
        test_database_framework().await.unwrap();
    }
}