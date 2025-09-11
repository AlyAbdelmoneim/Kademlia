use rusqlite::Connection;
use std::{io::Result};

pub trait Storage<V = String> {
    fn print(&self) -> Result<()>;
    fn store(&mut self, key: String, value: V) -> Result<()>;
    fn get(&self, key: String) -> Option<&V>;
    fn remove(&mut self, key: String) -> Result<()>;
    fn contains(&self, key: String) -> bool;
}

struct SqlLiteData {
    id: i32,
    value: String,
    updated_at: String,
}

pub struct SqlLiteStorage {
    // conn: rusqlite::Connection,
}

impl SqlLiteStorage {
    pub fn new(name: &str) -> Result<Self> {
        let conn = Self::init_db(name).unwrap();
        Ok(Self {})
    }

    fn init_db(name: &str) -> Result<rusqlite::Connection> {
        let conn = Connection::open(name).unwrap();
        // conn.execute(
        //         "CREATE TABLE IF NOT EXISTS my_table (
        //         id INTEGER PRIMARY KEY,
        //         value TEXT NOT NULL,
        //         updated_at DATETIME NOT NULL,
        //         UNIQUE(id)
        //     )",
        //     [],
        // ).unwrap();
        Ok(conn)
    }
}

impl Storage for SqlLiteStorage {
    fn print(&self) -> Result<()> {
        Ok(())
    }

    fn store(&mut self, key: String, value: String) -> Result<()> {
        Ok(())
    }

    fn get(&self, key: String) -> Option<&String> {
        static TEMP: String = String::new(); // Empty, not useful
        Some(&TEMP)
    }

    fn remove(&mut self, key: String) -> Result<()> {
        Ok(())
    }

    fn contains(&self, key: String) -> bool {
        false
    }
}
