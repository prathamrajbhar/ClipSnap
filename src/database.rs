use crate::models::{ContentType, HistoryEntry};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

/// Database layer wrapping a SQLite connection.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) the database at the given path.
    pub fn new(db_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create database dir: {:?}", parent))?;
        }

        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open database: {:?}", db_path))?;

        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;

        let db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Create an in-memory database (for testing).
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Create tables and indexes if they don't exist.
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS clipboard_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type TEXT NOT NULL,
                content_data BLOB,
                text_content TEXT,
                thumbnail BLOB,
                created_at INTEGER NOT NULL,
                file_size INTEGER,
                metadata TEXT,
                CHECK (
                    (content_type = 'image' AND content_data IS NOT NULL) OR
                    (content_type = 'text' AND text_content IS NOT NULL)
                )
            );

            CREATE INDEX IF NOT EXISTS idx_created_at
                ON clipboard_history(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_content_type
                ON clipboard_history(content_type);
            ",
        )?;
        log::info!("Database schema initialized");
        Ok(())
    }

    /// Insert an image entry. Returns the row id.
    pub fn insert_image(&self, png_bytes: &[u8], thumbnail: &[u8]) -> Result<i64> {
        let now = chrono::Utc::now().timestamp();
        let size = png_bytes.len() as i64;

        self.conn.execute(
            "INSERT INTO clipboard_history
                (content_type, content_data, thumbnail, created_at, file_size)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["image", png_bytes, thumbnail, now, size],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Insert a text entry. Returns the row id.
    pub fn insert_text(&self, text: &str) -> Result<i64> {
        let now = chrono::Utc::now().timestamp();
        let size = text.len() as i64;

        self.conn.execute(
            "INSERT INTO clipboard_history
                (content_type, text_content, created_at, file_size)
             VALUES (?1, ?2, ?3, ?4)",
            params!["text", text, now, size],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Retrieve the most recent entries filtered by type.
    pub fn get_recent_entries_by_type(&self, limit: usize, content_type: ContentType) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content_data, text_content,
                    thumbnail, created_at, file_size
             FROM clipboard_history
             WHERE content_type = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;

        let ct_str = content_type.to_str();
        let entries = stmt
            .query_map(params![ct_str, limit as i64], |row| {
                let ct_str: String = row.get(1)?;
                let content_type = ContentType::from_str(&ct_str).unwrap_or(ContentType::Text);
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    content_type,
                    image_data: row.get(2)?,
                    text_content: row.get(3)?,
                    thumbnail: row.get(4)?,
                    created_at: row.get(5)?,
                    file_size: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    /// Get a single entry by id (with full image data).
    pub fn get_entry(&self, id: i64) -> Result<Option<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content_data, text_content,
                    thumbnail, created_at, file_size
             FROM clipboard_history
             WHERE id = ?1",
        )?;

        let mut entries: Vec<HistoryEntry> = stmt
            .query_map(params![id], |row| {
                let ct_str: String = row.get(1)?;
                let content_type = ContentType::from_str(&ct_str).unwrap_or(ContentType::Text);
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    content_type,
                    image_data: row.get(2)?,
                    text_content: row.get(3)?,
                    thumbnail: row.get(4)?,
                    created_at: row.get(5)?,
                    file_size: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries.pop())
    }

    /// Search text entries by substring match.
    pub fn search_text(&self, query: &str) -> Result<Vec<HistoryEntry>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content_data, text_content,
                    thumbnail, created_at, file_size
             FROM clipboard_history
             WHERE text_content LIKE ?1
             ORDER BY created_at DESC
             LIMIT 50",
        )?;

        let entries = stmt
            .query_map(params![pattern], |row| {
                let ct_str: String = row.get(1)?;
                let content_type = ContentType::from_str(&ct_str).unwrap_or(ContentType::Text);
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    content_type,
                    image_data: row.get(2)?,
                    text_content: row.get(3)?,
                    thumbnail: row.get(4)?,
                    created_at: row.get(5)?,
                    file_size: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    /// Delete a single entry.
    #[allow(dead_code)]
    pub fn delete_entry(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Remove entries older than `days` days.
    pub fn cleanup_old_entries(&self, days: i64) -> Result<usize> {
        let cutoff = chrono::Utc::now().timestamp() - (days * 86400);
        let count = self.conn.execute(
            "DELETE FROM clipboard_history WHERE created_at < ?1",
            params![cutoff],
        )?;
        if count > 0 {
            log::info!("Cleaned up {} old clipboard entries", count);
        }
        Ok(count)
    }

    /// Clear clipboard history. If content_type is Some, only clear that type.
    pub fn clear_history(&self, content_type: Option<ContentType>) -> Result<usize> {
        let count = match content_type {
            Some(ct) => {
                let ct_str = ct.to_str();
                self.conn.execute("DELETE FROM clipboard_history WHERE content_type = ?1", params![ct_str])?
            }
            None => self.conn.execute("DELETE FROM clipboard_history", [])?,
        };
        Ok(count)
    }

    /// Enforce maximum entry count by deleting oldest entries.
    pub fn enforce_max_entries(&self, max: usize) -> Result<()> {
        self.conn.execute(
            "DELETE FROM clipboard_history WHERE id NOT IN (
                SELECT id FROM clipboard_history ORDER BY created_at DESC LIMIT ?1
            )",
            params![max as i64],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get_text() {
        let db = Database::new_in_memory().unwrap();
        let id = db.insert_text("hello world").unwrap();
        assert!(id > 0);

        let entries = db.get_recent_entries(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text_content.as_deref(), Some("hello world"));
        assert_eq!(entries[0].content_type, ContentType::Text);
    }

    #[test]
    fn test_insert_and_get_image() {
        let db = Database::new_in_memory().unwrap();
        let png = vec![0x89, 0x50, 0x4E, 0x47]; // Fake PNG header
        let thumb = vec![1, 2, 3];
        let id = db.insert_image(&png, &thumb).unwrap();
        assert!(id > 0);

        let entry = db.get_entry(id).unwrap().unwrap();
        assert_eq!(entry.content_type, ContentType::Image);
        assert_eq!(entry.image_data.unwrap(), png);
        assert_eq!(entry.thumbnail.unwrap(), thumb);
    }

    #[test]
    fn test_search_text() {
        let db = Database::new_in_memory().unwrap();
        db.insert_text("foo bar baz").unwrap();
        db.insert_text("hello world").unwrap();
        db.insert_text("foo qux").unwrap();

        let results = db.search_text("foo").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_delete_entry() {
        let db = Database::new_in_memory().unwrap();
        let id = db.insert_text("to delete").unwrap();
        db.delete_entry(id).unwrap();
        let entry = db.get_entry(id).unwrap();
        assert!(entry.is_none());
    }

    #[test]
    fn test_enforce_max_entries() {
        let db = Database::new_in_memory().unwrap();
        for i in 0..10 {
            db.insert_text(&format!("entry {}", i)).unwrap();
        }
        db.enforce_max_entries(5).unwrap();
        let entries = db.get_recent_entries(100).unwrap();
        assert_eq!(entries.len(), 5);
    }
}
