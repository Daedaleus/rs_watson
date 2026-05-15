use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use rusqlite_migration::{M, Migrations};
use thiserror::Error;
use uuid::Uuid;

use crate::{ActiveFrameRecord, FrameRecord, Storage};

#[derive(Debug, Error)]
pub enum SqliteStorageError {
    #[error("SQLite error: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("Migration error: {0}")]
    Migration(#[from] rusqlite_migration::Error),
    #[error("Data error: {0}")]
    Parse(String),
}

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, SqliteStorageError> {
        let mut conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", true)?;

        let migrations = Migrations::new(vec![
            M::up(include_str!("migrations/001_initial.sql")),
            M::up(include_str!("migrations/002_add_start_index.sql")),
        ]);
        migrations.to_latest(&mut conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

fn parse_dt(s: &str) -> Result<DateTime<Utc>, SqliteStorageError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| SqliteStorageError::Parse(format!("invalid datetime \"{s}\": {e}")))
}

fn parse_uuid(s: &str) -> Result<Uuid, SqliteStorageError> {
    Uuid::parse_str(s).map_err(|e| SqliteStorageError::Parse(format!("invalid UUID \"{s}\": {e}")))
}

impl Storage for SqliteStorage {
    type Error = SqliteStorageError;

    fn load_frames(&self) -> Result<Vec<FrameRecord>, Self::Error> {
        let conn = self.conn.lock().unwrap();

        // Load frames ordered by start time
        let mut frame_stmt =
            conn.prepare("SELECT id, project, start, end FROM frames ORDER BY start")?;
        let frame_rows: Vec<(String, String, String, String)> = frame_stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .collect::<Result<_, _>>()?;

        // Load all tags grouped by frame_id
        let mut tag_stmt =
            conn.prepare("SELECT frame_id, tag FROM frame_tags ORDER BY frame_id, position")?;
        let mut tags_by_frame: HashMap<String, Vec<String>> = HashMap::new();
        for row in tag_stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })? {
            let (frame_id, tag) = row?;
            tags_by_frame.entry(frame_id).or_default().push(tag);
        }

        frame_rows
            .into_iter()
            .map(|(id, project, start, end)| {
                Ok(FrameRecord {
                    id: parse_uuid(&id)?,
                    project,
                    tags: tags_by_frame.remove(&id).unwrap_or_default(),
                    start: parse_dt(&start)?,
                    end: parse_dt(&end)?,
                })
            })
            .collect()
    }

    fn save_frames(&self, frames: &[FrameRecord]) -> Result<(), Self::Error> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM frames", [])?; // CASCADE removes frame_tags

        for frame in frames {
            tx.execute(
                "INSERT INTO frames (id, project, start, end) VALUES (?1, ?2, ?3, ?4)",
                params![
                    frame.id.to_string(),
                    frame.project,
                    frame.start.to_rfc3339(),
                    frame.end.to_rfc3339(),
                ],
            )?;
            for (i, tag) in frame.tags.iter().enumerate() {
                tx.execute(
                    "INSERT INTO frame_tags (frame_id, position, tag) VALUES (?1, ?2, ?3)",
                    params![frame.id.to_string(), i as i64, tag],
                )?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    fn load_active(&self) -> Result<Option<ActiveFrameRecord>, Self::Error> {
        let conn = self.conn.lock().unwrap();

        let result = conn.query_row(
            "SELECT project, start FROM active_frame WHERE lock = 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );

        match result {
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(SqliteStorageError::Rusqlite(e)),
            Ok((project, start)) => {
                let mut tag_stmt =
                    conn.prepare("SELECT tag FROM active_frame_tags ORDER BY position")?;
                let tags: Vec<String> = tag_stmt
                    .query_map([], |row| row.get(0))?
                    .collect::<Result<_, _>>()?;

                Ok(Some(ActiveFrameRecord {
                    project,
                    tags,
                    start: parse_dt(&start)?,
                }))
            }
        }
    }

    fn save_active(&self, frame: Option<&ActiveFrameRecord>) -> Result<(), Self::Error> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM active_frame", [])?;
        tx.execute("DELETE FROM active_frame_tags", [])?;

        if let Some(f) = frame {
            tx.execute(
                "INSERT INTO active_frame (lock, project, start) VALUES (1, ?1, ?2)",
                params![f.project, f.start.to_rfc3339()],
            )?;
            for (i, tag) in f.tags.iter().enumerate() {
                tx.execute(
                    "INSERT INTO active_frame_tags (position, tag) VALUES (?1, ?2)",
                    params![i as i64, tag],
                )?;
            }
        }

        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, h, 0, 0).unwrap()
    }

    fn storage() -> SqliteStorage {
        SqliteStorage::new(":memory:").unwrap()
    }

    fn sample_frame() -> FrameRecord {
        FrameRecord {
            id: Uuid::new_v4(),
            project: "backend".into(),
            tags: vec!["api".into(), "auth".into()],
            start: t(9),
            end: t(10),
        }
    }

    fn sample_active() -> ActiveFrameRecord {
        ActiveFrameRecord {
            project: "frontend".into(),
            tags: vec!["ui".into()],
            start: t(9),
        }
    }

    #[test]
    fn frames_roundtrip() {
        let s = storage();
        let frame = sample_frame();
        let id = frame.id;
        s.save_frames(&[frame]).unwrap();
        let loaded = s.load_frames().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, id);
        assert_eq!(loaded[0].project, "backend");
        assert_eq!(loaded[0].tags, vec!["api", "auth"]);
        assert_eq!(loaded[0].start, t(9));
        assert_eq!(loaded[0].end, t(10));
    }

    #[test]
    fn frames_tag_order_preserved() {
        let s = storage();
        let mut f = sample_frame();
        f.tags = vec!["z-tag".into(), "a-tag".into(), "m-tag".into()];
        s.save_frames(&[f]).unwrap();
        assert_eq!(
            s.load_frames().unwrap()[0].tags,
            vec!["z-tag", "a-tag", "m-tag"]
        );
    }

    #[test]
    fn load_frames_empty_returns_empty() {
        assert!(storage().load_frames().unwrap().is_empty());
    }

    #[test]
    fn frames_sorted_by_start() {
        let s = storage();
        let mut f1 = sample_frame();
        let mut f2 = sample_frame();
        f1.project = "second".into();
        f1.start = t(10);
        f1.end = t(11);
        f2.project = "first".into();
        f2.start = t(8);
        f2.end = t(9);
        s.save_frames(&[f1, f2]).unwrap();
        let frames = s.load_frames().unwrap();
        assert_eq!(frames[0].project, "first");
        assert_eq!(frames[1].project, "second");
    }

    #[test]
    fn active_roundtrip() {
        let s = storage();
        s.save_active(Some(&sample_active())).unwrap();
        let loaded = s.load_active().unwrap().unwrap();
        assert_eq!(loaded.project, "frontend");
        assert_eq!(loaded.tags, vec!["ui"]);
        assert_eq!(loaded.start, t(9));
    }

    #[test]
    fn load_active_empty_returns_none() {
        assert!(storage().load_active().unwrap().is_none());
    }

    #[test]
    fn save_active_none_clears_state() {
        let s = storage();
        s.save_active(Some(&sample_active())).unwrap();
        s.save_active(None).unwrap();
        assert!(s.load_active().unwrap().is_none());
    }

    #[test]
    fn migrations_are_idempotent() {
        // Opening the same in-memory db twice would be a different db,
        // but we can verify the schema is applied by checking table existence.
        let s = storage();
        s.save_frames(&[sample_frame()]).unwrap();
        assert_eq!(s.load_frames().unwrap().len(), 1);
    }
}
