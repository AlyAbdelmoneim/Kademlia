use rusqlite::{Connection, OptionalExtension, Result, params};

pub fn connect(db_file: &str) -> Result<()> {
    let conn = Connection::open(db_file)?;
    init_tables(&conn)?;
    Ok(())
}

fn init_tables(connection: &Connection) -> Result<()> {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS data (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub fn get_value(connection: &Connection, key: &String) -> Result<Option<String>> {
    connection
        .query_row(
            "SELECT value FROM data WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
}

pub fn store_pair(connection: &Connection, key: &String, value: &String) -> Result<()> {
    let num = connection.execute(
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

pub fn update_pair(connection: &Connection, key: &String, value: &String) -> Result<()> {
    connection.execute(
        "update data set value = ?2 where key = ?1",
        params![key, value],
    )?;
    Ok(())
}

pub fn delete_pair(connection: &Connection, key: &String) -> Result<usize> {
    let num_of_rows = connection.execute("DELETE FROM data WHERE key = ?1", params![key])?;
    Ok(num_of_rows)
}
