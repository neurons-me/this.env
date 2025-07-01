//this.env/env/src/env/structs.rs
// by suiGn
//! Contains the core data structures and types used in the `this.env` crate.
use chrono::{DateTime, Utc};
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
    Approved,
    /// The environment has been explicitly blocked.
    Blocked(String),
    /// The environment has no endorsements and is pending approval.
    PendingApproval(String),
}
