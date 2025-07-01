//this.env/crate/src/middleware/domain_utils.rs
/// Extracts the root domain from a full host (e.g., "admin.example.com" → "example.com").
pub fn extract_root_domain(host: &str) -> Option<String> {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 2 {
        Some(format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]))
    } else {
        None
    }
}

/// Determines the parent domain if this is a subdomain.
pub fn determine_parent(host: &str) -> Option<String> {
    if let Some(root) = extract_root_domain(host) {
        if host != root {
            Some(root)
        } else {
            None
        }
    } else {
        None
    }
}

/// Returns (root_domain, subdomain) where subdomain is None if host == root.
pub fn split_host(host: &str) -> (String, Option<String>) {
    let root = extract_root_domain(host).unwrap_or_else(|| host.to_string());
    if host == root {
        (root, None)
    } else {
        let sub = host.trim_end_matches(&format!(".{}", &root)).to_string();
        (root, Some(sub))
    }
}

/// Quick check whether this request originates from localhost.
pub fn is_localhost(host: &str) -> bool {
    host.starts_with("localhost") || host.starts_with("127.0.0.1")
}