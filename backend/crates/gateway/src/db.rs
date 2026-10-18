use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use domain::{EventId, SlotBitmap, UserId};
use rusqlite::types::Type;
use rusqlite::{Connection, ErrorCode, OptionalExtension, params};

/// Opens (or creates) the gateway database and applies the schema.
///
/// `nick_key` is the lowercased nick. It backs the case-insensitive
/// uniqueness of nicks within an event: SQLite's `NOCASE` only folds ASCII,
/// so `Łukasz` and `łukasz` would otherwise be two different people.
pub fn open_database(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -64;
         PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS events (
             id          TEXT PRIMARY KEY,
             name        TEXT NOT NULL,
             slots       BLOB NOT NULL,
             created_at  INTEGER NOT NULL
         );

         CREATE TABLE IF NOT EXISTS participants (
             id            TEXT PRIMARY KEY,
             event_id      TEXT NOT NULL REFERENCES events(id),
             nick          TEXT NOT NULL,
             nick_key      TEXT NOT NULL,
             password_hash TEXT,
             created_at    INTEGER NOT NULL,
             UNIQUE (event_id, nick_key)
         );",
    )?;
    Ok(conn)
}

pub struct Event {
    pub name: String,
    pub slots: SlotBitmap,
}

pub struct Participant {
    pub id: UserId,
    pub nick: String,
    /// `None` means the nick is not password-protected.
    pub password_hash: Option<String>,
}

pub fn insert_event(
    conn: &Connection,
    id: &EventId,
    name: &str,
    slots: &SlotBitmap,
) -> rusqlite::Result<()> {
    conn.prepare_cached(
        "INSERT INTO events (id, name, slots, created_at) VALUES (?1, ?2, ?3, ?4)",
    )?
    .execute(params![id.as_str(), name, slots.as_bytes(), unix_now()])?;
    Ok(())
}

pub fn find_event(conn: &Connection, id: &EventId) -> rusqlite::Result<Option<Event>> {
    conn.prepare_cached("SELECT name, slots FROM events WHERE id = ?1")?
        .query_row(params![id.as_str()], |row| {
            let slots: Vec<u8> = row.get(1)?;
            Ok(Event {
                name: row.get(0)?,
                slots: SlotBitmap::from_bytes(&slots).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(1, Type::Blob, Box::new(e))
                })?,
            })
        })
        .optional()
}

pub fn find_participant(
    conn: &Connection,
    event_id: &EventId,
    nick_key: &str,
) -> rusqlite::Result<Option<Participant>> {
    conn.prepare_cached(
        "SELECT id, nick, password_hash FROM participants
          WHERE event_id = ?1 AND nick_key = ?2",
    )?
    .query_row(params![event_id.as_str(), nick_key], |row| {
        let id: String = row.get(0)?;
        Ok(Participant {
            id: UserId::parse(&id).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(0, Type::Text, Box::new(e))
            })?,
            nick: row.get(1)?,
            password_hash: row.get(2)?,
        })
    })
    .optional()
}

/// Inserts a participant. Returns `Ok(false)` when the nick was taken in
/// the meantime by a concurrent join.
pub fn insert_participant(
    conn: &Connection,
    event_id: &EventId,
    participant: &Participant,
    nick_key: &str,
) -> rusqlite::Result<bool> {
    let result = conn
        .prepare_cached(
            "INSERT INTO participants (id, event_id, nick, nick_key, password_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?
        .execute(params![
            participant.id.as_str(),
            event_id.as_str(),
            participant.nick,
            nick_key,
            participant.password_hash,
            unix_now(),
        ]);
    match result {
        Ok(_) => Ok(true),
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.code == ErrorCode::ConstraintViolation =>
        {
            Ok(false)
        }
        Err(error) => Err(error),
    }
}

pub fn ping(conn: &Connection) -> rusqlite::Result<()> {
    conn.query_row("SELECT 1", [], |_| Ok(()))
}

fn unix_now() -> i64 {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock is after 1970")
        .as_secs();
    i64::try_from(seconds).expect("unix time fits in i64")
}
