//this.env/src/env/methods/resolve.rs
// by suiGn
// This file contains the implementation of the `resolve_from_request` method for the `Env`
// struct, which is responsible for determining the environment configuration based on an incoming request.
// It handles the logic of checking if an environment already exists in the database, and if not,
// it defaults to an approved state while logging the request details.
use crate::env::structs::{EnvRequestLog, EnvStatus};
use crate::middleware::env_request::EnvRequestInfo;
use crate::middleware::env_request::EnvRequest;
use crate::env::Env;
use rusqlite::{params, Connection, Result as SqlResult};

impl Env {
    fn check_if_blocked(req: &EnvRequest) -> Option<String> {
        // Ejemplo simple: bloquear por dominio
        let blocked_domains = vec!["malicious.com", "banned.example"];
        let domain = match req {
            EnvRequest::Http(http_req) => http_req.headers.get("domain"),
            EnvRequest::Ws(ws_req) => ws_req.headers.get("domain"),
            EnvRequest::Cli(_) => None,
        };

        if let Some(domain) = domain {
            if blocked_domains.contains(&domain.as_str()) {
                return Some(format!("Domain '{}' is blocked", domain));
            }
        }

        None
    }

    pub fn resolve(req: &EnvRequest, conn: &Connection) -> EnvStatus {
        log::debug!("this.env resolve: attempting to fetch env from request");
        // Extract domain and env_type from EnvRequest variants
        let (domain, _env_type) = match req {
            EnvRequest::Http(http_req) => {
                // Assuming domain and env_type are headers or fields in http_req
                // Replace the following with actual extraction logic
                let domain = http_req.headers.get("domain").cloned().unwrap_or_default();
                let env_type = http_req.headers.get("env_type").cloned().unwrap_or_default();
                (domain, env_type)
            }
            EnvRequest::Ws(ws_req) => {
                // Assuming domain and env_type are fields or headers in ws_req
                let domain = ws_req.headers.get("domain").cloned().unwrap_or_default();
                let env_type = ws_req.headers.get("env_type").cloned().unwrap_or_default();
                (domain, env_type)
            }
            EnvRequest::Cli(_cli_req) => {
                let domain = "localhost".to_string();
                let env_type = "cli".to_string();
                (domain, env_type)
            }
            // Add other EnvRequest variants here if any
        };

        // Dummy lookup logic; replace with actual query
        let env_found = false;
        if let Some(reason) = Self::check_if_blocked(req) {
            log::debug!("this.env resolve: request is explicitly blocked");
            // Convert EnvRequest to EnvRequestLog for logging
            let mut log_entry: EnvRequestLog = req.clone().into();
            log_entry.decision = "Blocked".into();
            log_entry.reason = reason.clone();
            // Save the log
            if let Err(e) = Self::save_log_to_sqlite(conn, &log_entry) {
                log::error!("this.env resolve: failed to save blocked log: {:?}", e);
            } else {
                log::debug!("this.env resolve: blocked log saved to SQLite");
            }
            let env_info = EnvRequestInfo::from(req);
            return EnvStatus::Blocked { env_request: env_info, reason };
        }

        if env_found {
            log::debug!("this.env resolve: found existing env");
            // TODO: return fetched env
            let env_info = EnvRequestInfo::from(req);
            EnvStatus::Approved { env_request: env_info }
        } else {
            log::debug!("this.env resolve: no env found, defaulting to Pending");
            // Convert EnvRequest to EnvRequestLog for logging
            let mut log_entry: EnvRequestLog = req.clone().into();
            log_entry.decision = "PendingApproval".into();
            log_entry.reason = "No existing env found".into();

            // Save the log
            if let Err(e) = Self::save_log_to_sqlite(conn, &log_entry) {
                log::error!("this.env resolve: failed to save log: {:?}", e);
            } else {
                log::debug!("this.env resolve: log saved to SQLite");
            }
            let env_info = EnvRequestInfo::from(req);
            EnvStatus::PendingApproval { env_request: env_info, reason: domain }
        }
    }

    fn save_log_to_sqlite(conn: &Connection, log: &EnvRequestLog) -> SqlResult<()> {
        conn.execute(
            "INSERT INTO env_request_logs (ip, method, path, host, headers, decision, reason, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                log.ip.as_deref().unwrap_or("unknown"),
                log.method,
                log.path,
                log.host,
                log.headers,
                log.decision,
                log.reason,
                log.timestamp,
            ],
        )?;
        Ok(())
    }
}