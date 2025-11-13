//this.env/env/src/env/env.rs
// by suiGn
//! Contains the main `Env` structure and its methods for managing environment configurations.
use std::collections::HashMap;
use std::path::Path;
use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};
use rusqlite::OptionalExtension;
use crate::env::error::EnvError;
type Result<T> = std::result::Result<T, EnvError>;
use crate::env::structs::{Endorsement, EnvType, TrustLevel, RouteInfo, EnvStatus};
use crate::middleware::env_request::EnvRequest;
use crate::middleware::env_request::EnvRequestInfo;
use crate::utils::domain_utils::{determine_parent, split_host};
use crate::utils::id_utils::generate_id_for;
use crate::env::migrate_schema::migrate_schema;
/// Main structure representing an environment configuration.
/// # Examples
/// ```
/// use this_env::env::{Env, EnvType, TrustLevel};
/// let env = Env::new(
///     "example.com".into(),
///     EnvType::RemoteWeb,
///     TrustLevel::Medium,
///     endorsements: Vec::new(),
///     routes: HashMap::new(),
///     parent: None,
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Env {
    /// Human-readable domain name (e.g., "neurons.me")
    pub domain: String,
    /// A unique identifier or fingerprint for this environment domain
    pub id: String,
    /// The type of environment domain (e.g., Localhost, RemoteWeb, etc.)
    pub env_type: EnvType,
    /// The trust level assigned to this environment domain
    pub trust: TrustLevel,
    // Metadata now lives in SQLite, no in-memory copy needed
    /// Route-specific metadata and activity tracking
    pub routes: HashMap<String, RouteInfo>,
    /// Optional parent domain, used to express subdomain hierarchy (e.g., admin.example.com → example.com)
    pub parent: Option<String>,
}

impl Env {
    /// Instantiates a new `Env` object.
    pub fn new(domain: String, env_type: EnvType, trust: TrustLevel) -> Self {
        let id = generate_id_for(&domain);
        Self {
            parent: determine_parent(&domain),
            domain,
            id,
            env_type,
            trust,
            routes: HashMap::new(),
        }
    }

/// Initializes a SQLite database to persist this environment's data.
  pub fn init_sqlite(&self, base_path: &Path) -> Result<Connection> {
      {
          use std::fs;
          let db_path = base_path.join(".db");
          if let Some(parent) = db_path.parent() {
              fs::create_dir_all(parent)?; // io::Error -> EnvError via #[from]
          }
          let conn = Connection::open(db_path)?; // rusqlite::Error -> EnvError via #[from]
          migrate_schema(&conn)?;                // rusqlite::Error -> EnvError via #[from]
          Ok(conn)
      }
}
    /// Loads the environment from SQLite based on the domain.
    /// Returns `None` if not found.
    fn load(conn: &Connection, domain: &str) -> Result<Option<Self>> {
        Ok(conn
            .query_row(
                "SELECT id, env_type, trust, parent FROM env WHERE domain = ?1",
                params![domain],
                |row| {
                    Ok(Env {
                        domain: domain.into(),
                        id: row.get(0)?,
                        env_type: row.get(1)?,
                        trust: row.get(2)?,
                        parent: row.get(3)?,
                        routes: HashMap::new(),
                    })
                },
            )
            .optional()?)
    }

/// Constructs an `Env` instance from a normalized `EnvRequest`.
/// Internally uses the host to determine the root domain and fetches the environment.
pub fn from_request(conn: &Connection, req: &EnvRequest) -> Result<Option<Self>> {
    let host = match req {
        EnvRequest::Http(r) => &r.host,
        EnvRequest::Ws(r) => &r.host,
        EnvRequest::Cli(_) => return Ok(None),
    };
    let (root, _) = split_host(host);
    Self::load(conn, &root)
}
    /// Evaluates and returns the current endorsement status of the environment.
    /// If endorsements exist, it checks for any positive approval.
    /// If no endorsements are present, it defaults to PendingApproval status.
    pub fn status(&self, conn: &Connection) -> Result<EnvStatus> {
        let endorsements = Self::get_endorsements(conn, &self.domain)?;
        let dummy_req = EnvRequest::Cli(Default::default());
        if endorsements.is_empty() {
            return Ok(EnvStatus::PendingApproval {
                env_request: EnvRequestInfo::from(&dummy_req),
                reason: self.domain.clone(),
            });
        }
        let has_approval = endorsements.iter().any(|e| e.approved);
        let has_rejection = endorsements.iter().any(|e| !e.approved);
        if has_approval {
            Ok(EnvStatus::Approved {
                env_request: EnvRequestInfo::from(&dummy_req),
            })
        } else if has_rejection {
            Ok(EnvStatus::Blocked {
                env_request: EnvRequestInfo::from(&dummy_req),
                reason: self.domain.clone(),
            })
        } else {
            Ok(EnvStatus::PendingApproval {
                env_request: EnvRequestInfo::from(&dummy_req),
                reason: self.domain.clone(),
            })
        }
    }
    /// Sets a metadata key-value pair in the SQLite database.
    pub fn set_metadata_sql(conn: &Connection, domain: &str, key: &str, value: &str) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO metadata (domain, key, value) VALUES (?1, ?2, ?3)",
            params![domain, key, value],
        )?;
        Ok(())}
    /// Retrieves a metadata value by key from the SQLite database.
    pub fn get_metadata_sql(conn: &Connection, domain: &str, key: &str) -> Result<Option<String>> {
        Ok(conn
            .query_row(
                "SELECT value FROM metadata WHERE domain = ?1 AND key = ?2",
                params![domain, key],
                |row| row.get(0),
            )
            .optional()?)
    }

    /// Adds or updates an endorsement in the SQLite database.
    pub fn add_endorsement(conn: &Connection, domain: &str, endorsement: Endorsement) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO endorsements (domain, endorser, approved, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![domain, endorsement.endorser, endorsement.approved as i32, endorsement.timestamp],
        )?;
        Ok(())
    }
    /// Retrieves all endorsements for a given domain from the SQLite database.
    pub fn get_endorsements(conn: &Connection, domain: &str) -> Result<Vec<Endorsement>> {
        let mut stmt = conn.prepare(
            "SELECT endorser, approved, timestamp FROM endorsements WHERE domain = ?1"
        )?;
        let rows = stmt.query_map(params![domain], |row| {
            Ok(Endorsement {
                endorser: row.get(0)?,
                approved: row.get::<_, i32>(1)? != 0,
                timestamp: row.get(2)?,
            })
        })?;

        let mut endorsements = Vec::new();
        for endorsement in rows {
            endorsements.push(endorsement?);
        }
        Ok(endorsements)
    }
    /// Checks if a specific endorser has approved the domain.
    pub fn is_endorsed_by(conn: &Connection, domain: &str, endorser: &str) -> Result<bool> {
        let val = conn
            .query_row(
                "SELECT approved FROM endorsements WHERE domain = ?1 AND endorser = ?2",
                params![domain, endorser],
                |row| {
                    let approved: i32 = row.get(0)?;
                    Ok(approved != 0)
                },
            )
            .optional()?
            .unwrap_or(false);
        Ok(val)
    }

    /// Retrieves the parent environment of this domain, if defined.
    /// Returns `None` if there is no parent or the parent is not found in the database.
    pub fn get_parent(&self, conn: &Connection) -> Result<Option<Env>> {
        if let Some(ref parent_domain) = self.parent {
            Env::load(conn, parent_domain)
        } else {
            Ok(None)
        }
    }

    /// Retrieves all child environments that reference this environment as their parent.
    /// Returns an empty list if there are no children.
    pub fn get_children(&self, conn: &Connection) -> Result<Vec<Env>> {
        let mut stmt = conn.prepare(
            "SELECT domain, id, env_type, trust, parent FROM env WHERE parent = ?1"
        )?;
        let rows = stmt.query_map(params![&self.domain], |row| {
            Ok(Env {
                domain: row.get(0)?,
                id: row.get(1)?,
                env_type: row.get(2)?,
                trust: row.get(3)?,
                parent: row.get(4)?,
                routes: HashMap::new(),
            })
        })?;

        let mut children = Vec::new();
        for env in rows {
            children.push(env?);
        }
        Ok(children)
    }
}