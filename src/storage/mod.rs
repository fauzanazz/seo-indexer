mod models;
pub use models::*;

use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

pub struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    pub fn new(path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        Self::init_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        Self::init_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS submissions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                method TEXT NOT NULL,
                success INTEGER NOT NULL,
                message TEXT,
                submitted_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert(&self, record: &SubmissionRecord) -> Result<i64, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO submissions (url, method, success, message, submitted_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                record.url,
                record.method,
                record.success as i64,
                record.message,
                record.submitted_at.to_rfc3339(),
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_history(&self, limit: usize) -> Result<Vec<SubmissionRecord>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, url, method, success, message, submitted_at
             FROM submissions
             ORDER BY submitted_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], row_to_record)?;
        rows.collect()
    }

    pub fn get_by_url(&self, url: &str) -> Result<Vec<SubmissionRecord>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, url, method, success, message, submitted_at
             FROM submissions
             WHERE url = ?1
             ORDER BY submitted_at DESC",
        )?;
        let rows = stmt.query_map(params![url], row_to_record)?;
        rows.collect()
    }
}

fn row_to_record(row: &rusqlite::Row) -> Result<SubmissionRecord, rusqlite::Error> {
    let submitted_at_str: String = row.get(5)?;
    let submitted_at = chrono::DateTime::parse_from_rfc3339(&submitted_at_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());

    Ok(SubmissionRecord {
        id: Some(row.get(0)?),
        url: row.get(1)?,
        method: row.get(2)?,
        success: row.get::<_, i64>(3)? != 0,
        message: row.get(4)?,
        submitted_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_record(url: &str, method: &str, success: bool) -> SubmissionRecord {
        SubmissionRecord {
            id: None,
            url: url.to_string(),
            method: method.to_string(),
            success,
            message: Some("ok".to_string()),
            submitted_at: Utc::now(),
        }
    }

    #[test]
    fn test_storage_insert_submission() {
        let storage = Storage::in_memory().unwrap();
        let record = sample_record("https://example.com/page", "indexnow", true);
        let id = storage.insert(&record).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_storage_query_history() {
        let storage = Storage::in_memory().unwrap();
        storage
            .insert(&sample_record("https://example.com/a", "indexnow", true))
            .unwrap();
        storage
            .insert(&sample_record("https://example.com/b", "ping", false))
            .unwrap();

        let history = storage.get_history(10).unwrap();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_storage_query_by_url() {
        let storage = Storage::in_memory().unwrap();
        let target = "https://example.com/target";
        storage
            .insert(&sample_record(target, "indexnow", true))
            .unwrap();
        storage
            .insert(&sample_record(target, "ping", true))
            .unwrap();
        storage
            .insert(&sample_record("https://example.com/other", "google", true))
            .unwrap();

        let results = storage.get_by_url(target).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.url == target));
    }
}
