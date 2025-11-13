// This file is part of the this.env crate.
// It provides methods to retrieve request logs for a specific environment.
// by suiGn
// La función get_request_logs ahora acepta un parámetro count para limitar la cantidad de logs devueltos. 
// Puedes pasar Some(50) para obtener los últimos 50, None para todos, o implementar lógica en frontend/backend para cargar más progresivamente (hot load con scroll). 
use rusqlite::{Connection, Result as SqlResult};
use crate::env::structs::EnvRequestLog;

pub fn get_request_logs(conn: &Connection, domain: Option<&str>, count: Option<usize>, offset: Option<usize>) -> SqlResult<Vec<EnvRequestLog>> {
    let query = match (domain, count) {
        (Some(_), Some(_)) => "SELECT method, path, host, timestamp, ip, headers, decision, reason, domain FROM env_request_logs WHERE domain = ? ORDER BY timestamp DESC LIMIT ? OFFSET ?",
        (Some(_), None) => "SELECT method, path, host, timestamp, ip, headers, decision, reason, domain FROM env_request_logs WHERE domain = ? ORDER BY timestamp DESC",
        (None, Some(_)) => "SELECT method, path, host, timestamp, ip, headers, decision, reason, domain FROM env_request_logs ORDER BY timestamp DESC LIMIT ? OFFSET ?",
        (None, None) => "SELECT method, path, host, timestamp, ip, headers, decision, reason, domain FROM env_request_logs ORDER BY timestamp DESC",
    };

    log::debug!("Fetching request logs with domain filter: {:?}, count: {:?}, offset: {:?}", domain, count, offset);
    let mut stmt = conn.prepare(query)?;

    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<EnvRequestLog> {
        Ok(EnvRequestLog {
            method: row.get(0)?,
            path: row.get(1)?,
            host: row.get(2)?,
            timestamp: row.get(3)?,
            ip: row.get(4)?,
            headers: row.get(5)?,
            decision: row.get(6)?,
            reason: row.get(7)?,
            domain: row.get(8)?,
        })
    };

    let rows = match (domain, count) {
        (Some(d), Some(c)) => stmt.query_map(rusqlite::params![d, c as i64, offset.unwrap_or(0) as i64], map_row)?,
        (Some(d), None) => stmt.query_map([d], map_row)?,
        (None, Some(c)) => stmt.query_map(rusqlite::params![c as i64, offset.unwrap_or(0) as i64], map_row)?,
        (None, None) => stmt.query_map([], map_row)?,
    };

    let mut logs = Vec::new();
    for log in rows {
        logs.push(log?);
    }
    log::debug!("Fetched {} total logs", logs.len());
    Ok(logs)
}