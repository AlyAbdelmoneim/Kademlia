use rusqlite::{Connection, OptionalExtension, params};

pub type StorageResult<T, E = StorageError> = Result<T, E>;

#[derive(Debug)]
pub struct StorageError {
    pub message: String
}

pub trait Storage<V = String> {
    fn print(&self) -> StorageResult<()>;
    fn store(&self, key: &str, value: &V) -> StorageResult<()>; //Note: this should me &mut self ideally if a storage will mutate its own in memory data, however, that will require we update the Arc and how we handle cloning the node and so on ... //probably use mutexes or something..
    fn get(&self, key: &str) -> StorageResult<Option<V>>;
    fn remove(&self, key: &str) -> StorageResult<()>;
    fn contains(&self, key: &str) -> StorageResult<bool>;
}

impl From<rusqlite::Error> for StorageError {
    fn from(error: rusqlite::Error) -> Self {
        StorageError {
            message: error.to_string()
        }
    }
}

impl From<StorageError> for std::io::Error {
    fn from(error: StorageError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, error.message)
    }
}

#[derive(Debug)]
pub struct SqlLiteStorage {
    db_name: String,
}

impl SqlLiteStorage {
    pub fn new(name: &str) -> StorageResult<Self> {
        Self::init_db(name)?;
        Ok(Self {
            db_name: name.to_string(),
        })
    }

    fn init_db(name: &str) -> StorageResult<()> {
        let conn = Connection::open(name)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS data (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
            [],
        )?;
        Ok(())
    }
}

impl Storage for SqlLiteStorage {
    fn print(&self) -> StorageResult<()> {
        Ok(())
    }

    fn store(&self, key: &str, value: &String) -> StorageResult<()> {
        let conn = Connection::open(self.db_name.clone())?;
        let num = conn.execute(
            "INSERT INTO data (key, value) VALUES (?1, ?2) 
            ON CONFLICT 
            DO
            UPDATE SET value = ?2
            WHERE key = ?1",
            params![key, value],
        )?;
        if num == 0 {
            println!("didn't insert");
        } else {
            println!("inserted successfully");
        }
        Ok(())
    }

    fn get(&self, key: &str) -> StorageResult<Option<String>> {
        let conn = Connection::open(self.db_name.clone())?;
        Ok(conn.query_row(
            "SELECT value FROM data WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?)
    }

    fn remove(&self, key: &str) -> StorageResult<()> {
        let conn = Connection::open(self.db_name.clone())?;
        let num_of_rows = conn.execute("DELETE FROM data WHERE key = ?1", params![key])?;
        if num_of_rows > 0 {
            println!("deleted the pair of key : {} ", key);
        } else {
            println!("no such key");
        }
        Ok(())
    }

    fn contains(&self, key: &str) -> StorageResult<bool> {
        self.get(key).map(|opt| opt.is_some())
    }
}

