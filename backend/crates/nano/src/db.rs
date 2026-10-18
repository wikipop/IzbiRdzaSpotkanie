use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use domain::State;
use proto::AvailabilityEntry;
use rusqlite::types::Type;
use rusqlite::{Connection, params};

/// Opens (or creates) the slot database and applies the schema.
///
/// The pragmas favour a small memory footprint: WAL keeps writes cheap,
/// a 64 KiB page cache is plenty for a table this small, and `mmap` stays
/// off so the database is never mapped into the process' memory.
pub fn open_database(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -64;

         CREATE TABLE IF NOT EXISTS availability (
             event_id    TEXT NOT NULL,
             user_id     TEXT NOT NULL,
             nick        TEXT NOT NULL,
             state       TEXT NOT NULL CHECK (state IN ('yes', 'maybe')),
             updated_at  INTEGER NOT NULL,
             PRIMARY KEY (event_id, user_id)
         );",
    )?;
    Ok(conn)
}

pub fn list_entries(conn: &Connection, event_id: &str) -> rusqlite::Result<Vec<AvailabilityEntry>> {
    let mut statement = conn.prepare_cached(
        "SELECT user_id, nick, state, updated_at
           FROM availability
          WHERE event_id = ?1
          ORDER BY updated_at, user_id",
    )?;
    let rows = statement.query_map(params![event_id], |row| {
        let state: String = row.get(2)?;
        let state: State = state
            .parse()
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, Type::Text, Box::new(e)))?;
        Ok(AvailabilityEntry {
            user_id: row.get(0)?,
            nick: row.get(1)?,
            state: state.into(),
            updated_at: row.get(3)?,
        })
    })?;
    rows.collect()
}

/// Inserts or replaces the participant's entry and returns it.
pub fn upsert_entry(
    conn: &Connection,
    event_id: &str,
    user_id: &str,
    nick: &str,
    state: State,
) -> rusqlite::Result<AvailabilityEntry> {
    let updated_at = unix_now();
    conn.prepare_cached(
        "INSERT INTO availability (event_id, user_id, nick, state, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (event_id, user_id) DO UPDATE
            SET nick = excluded.nick,
                state = excluded.state,
                updated_at = excluded.updated_at",
    )?
    .execute(params![event_id, user_id, nick, state.as_str(), updated_at])?;
    Ok(AvailabilityEntry {
        user_id: user_id.to_owned(),
        nick: nick.to_owned(),
        state: state.into(),
        updated_at,
    })
}

/// Deletes the participant's entry; deleting a missing entry is not an error.
pub fn delete_entry(conn: &Connection, event_id: &str, user_id: &str) -> rusqlite::Result<()> {
    conn.prepare_cached("DELETE FROM availability WHERE event_id = ?1 AND user_id = ?2")?
        .execute(params![event_id, user_id])?;
    Ok(())
}

/// Liveness probe of the database used by `/health`.
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
