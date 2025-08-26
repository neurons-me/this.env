//this.env/env/src/env/structs.rs
// by suiGn
//! Contains the core data structures and types used in the `this.env` crate.
use chrono::{DateTime, Utc};
use crate::middleware::env_request::EnvRequestInfo;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
/// Endorser signature and approval status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endorsement {
    pub endorser: String,
    pub approved: bool,
    pub timestamp: u64,
}

/// Represents the type of environment where an identity operates.
///
/// # Examples
///
/// ```
/// use this_env::env::EnvType;
/// let t = EnvType::Localhost;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvType {
    Localhost,
    RemoteWeb,
    Extension,
    Desktop,
    P2P,
}

/// Represents the level of trust assigned to an environment.
///
/// # Examples
///
/// ```
/// use this_env::env::TrustLevel;
/// let trust = TrustLevel::High;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel { 
    Untrusted,
    Low,
    Medium,
    High,
    Full,
}

/// Represents metadata and analytics for a specific route within an environment.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteInfo {
    /// Number of times this route was accessed.
    pub hit_count: u64,
    /// Timestamp of the last time this route was accessed.
    pub last_seen: DateTime<Utc>,
    /// Key-value metadata for this specific route.
    pub metadata: HashMap<String, String>,
}

// Implement ToSql and FromSql for EnvType and TrustLevel
use rusqlite::{ToSql, Result as SqlResult};
use rusqlite::types::{FromSql, FromSqlError, ValueRef, ToSqlOutput, Value};
impl ToSql for EnvType {
    fn to_sql(&self) -> SqlResult<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Text(match self {
            EnvType::Localhost => "Localhost",
            EnvType::RemoteWeb => "RemoteWeb",
            EnvType::Extension => "Extension",
            EnvType::Desktop => "Desktop",
            EnvType::P2P => "P2P",
        }.to_string())))
    }
}

impl FromSql for EnvType {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value.as_str()? {
            "Localhost" => Ok(EnvType::Localhost),
            "RemoteWeb" => Ok(EnvType::RemoteWeb),
            "Extension" => Ok(EnvType::Extension),
            "Desktop" => Ok(EnvType::Desktop),
            "P2P" => Ok(EnvType::P2P),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for TrustLevel {
    fn to_sql(&self) -> SqlResult<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Text(match self {
            TrustLevel::Untrusted => "Untrusted",
            TrustLevel::Low => "Low",
            TrustLevel::Medium => "Medium",
            TrustLevel::High => "High",
            TrustLevel::Full => "Full",
        }.to_string())))
    }
}


impl FromSql for TrustLevel {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value.as_str()? {
            "Untrusted" => Ok(TrustLevel::Untrusted),
            "Low" => Ok(TrustLevel::Low),
            "Medium" => Ok(TrustLevel::Medium),
            "High" => Ok(TrustLevel::High),
            "Full" => Ok(TrustLevel::Full),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

/// Represents the endorsement evaluation result for a given environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvStatus {
    /// The environment has been approved by at least one endorser.
    Approved {
        env_request: EnvRequestInfo,
    },
    /// The environment has been explicitly blocked.
    Blocked {
        env_request: EnvRequestInfo,
        reason: String,
    },
    /// The environment has no endorsements and is pending approval.
    PendingApproval {
        env_request: EnvRequestInfo,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvRequestLog {
    pub timestamp: String,
    pub method: String,
    pub path: String,
    pub ip: Option<String>,
    pub host: String,
    pub headers: String,
    pub decision: String,
    pub reason: String,
    pub domain: String,
}


use crate::middleware::env_request::EnvRequest;
impl From<EnvRequest> for EnvRequestLog {
    fn from(req: EnvRequest) -> Self {
        match req {
            EnvRequest::Http(http) => Self {
                timestamp: Utc::now().to_rfc3339(),
                method: http.method,
                path: http.path,
                ip: http.ip,
                host: http.host.clone(),
                headers: serde_json::to_string(&http.headers).unwrap_or_default(),
                decision: String::new(),
                reason: String::new(),
                domain: http.headers.get("domain").cloned().unwrap_or_else(|| http.host.clone()),
            },
            EnvRequest::Ws(ws) => Self {
                timestamp: Utc::now().to_rfc3339(),
                method: "WS".into(),
                path: "".into(),
                ip: ws.ip,
                host: ws.host.clone(),
                headers: serde_json::to_string(&ws.headers).unwrap_or_default(),
                decision: String::new(),
                reason: String::new(),
                domain: ws.headers.get("domain").cloned().unwrap_or_else(|| ws.host.clone()),
            },
            EnvRequest::Cli(_) => Self {
                timestamp: Utc::now().to_rfc3339(),
                method: "CLI".into(),
                path: "".into(),
                ip: None,
                host: "localhost".into(),
                headers: "{}".into(),
                decision: String::new(),
                reason: String::new(),
                domain: "localhost".into(),
            },
        }
    }
}
