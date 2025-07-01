//this.env/crate/src/middleware/actix/config.rs
//by suiGn
/// Runtime behaviour flags for the middleware.
#[derive(Debug, Clone)]
pub struct ActixMwConfig {
    /// If `true`, requests whose EnvStatus is *PendingApproval* will be let through.
    pub allow_pending: bool,
    /// If `true`, requests whose EnvStatus is *Blocked* will be let through.
    pub allow_blocked: bool,
    /// If `true`, the middleware will prefer returning HTML even when the client
    /// doesn’t explicitly ask for it (handy for browsers).  
    /// If `false`, falls back to plain‑text/JSON unless the `Accept` header contains `text/html`.
    pub prefer_html: bool,
    /// If `true`, the middleware won’t do any automatic decision – it will *always*
    /// call the inner service and merely log the EnvStatus.  
    /// This lets the application layer decide what to do.
    pub manual_mode: bool,
}

impl Default for ActixMwConfig {
    fn default() -> Self {
        Self {
            allow_pending: false,
            allow_blocked: false,
            prefer_html: false,
            manual_mode: false,
        }
    }
}