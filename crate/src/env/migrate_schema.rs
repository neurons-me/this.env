//this.env/crate/src/env/migrate_schema.rs by suiGn
// this is the place to define and migrate the database schema of this.env
use rusqlite::{Connection, Result as SqlResult};
pub fn migrate_schema(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS env (
            domain TEXT PRIMARY KEY,
            id TEXT NOT NULL,
            env_type TEXT NOT NULL,
            trust TEXT NOT NULL,
            parent TEXT
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS metadata (
            domain TEXT,
            key TEXT,
            value TEXT,
            PRIMARY KEY (domain, key)
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS routes (
            domain TEXT,
            path TEXT,
            hit_count INTEGER,
            last_seen TEXT,
            PRIMARY KEY (domain, path)
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS route_metadata (
            domain TEXT,
            path TEXT,
            key TEXT,
            value TEXT,
            PRIMARY KEY (domain, path, key)
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS endorsements (
            domain TEXT,
            endorser TEXT,
            approved INTEGER,
            timestamp INTEGER,
            PRIMARY KEY (domain, endorser)
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS env_request_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            method TEXT,
            path TEXT,
            ip TEXT,
            host TEXT,
            headers TEXT,
            decision TEXT,
            reason TEXT
        )",
        [],
    )?;
    Ok(())
}