//this.env/crate/src/middleware/actix/utils.rs
// by suiGn
//! Helper utilities for actix middleware processing.
use std::collections::HashMap;
use actix_web::http::header::HeaderMap;
pub fn headers_to_hashmap(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|val| (k.to_string(), val.to_string())))
        .collect()
}