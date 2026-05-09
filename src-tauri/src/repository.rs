use rusqlite::{Connection, Result as SqlResult};
use std::sync::Mutex;
use crate::models::*;

pub struct Repository {
    conn: Mutex<Connection>,
}
impl Repository {
    pub fn new() -> SqlResult<Self> {
        let conn = Connection::open("exercise_tracker.db")?;
        
        conn.execute_batch(include_str!("schema.sql"))?;
        
        Ok(Repository { conn: Mutex::new(conn) })
    }

    pub fn create_exercise(&self, ex: &mut Exercise) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO exercises (id, name, category, notes) VALUES (?1, ?2, ?3, ?4)",
            (&ex.id, &ex.name, &ex.category, &ex.notes),
        )?;
        Ok(())
    }

    // Add more methods later (log workout, get history, etc.)
}