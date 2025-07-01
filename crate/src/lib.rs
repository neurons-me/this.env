//this.env/crate/src/lib.rs
//by suiGn
//! Entry point for the this.env crate.
//! Exposes core environment management and middleware logic.
pub mod env;
pub mod middleware;
pub mod utils;
// Expose Actix middleware with camelCase alias for convenience
pub use middleware::actix::ActixMiddleware as actixMiddleware;
