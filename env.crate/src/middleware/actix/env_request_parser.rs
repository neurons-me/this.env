//this.env/crate/src/middleware/actix/env_request_parser.rs
// by suiGn
// Module for parsing Actix `HttpRequest` into `EnvRequest`
// This module provides a function to convert Actix's `HttpRequest` into an internal
// `EnvRequest` enum, which is used for environment analysis and routing.
// It handles both HTTP and WebSocket requests, extracting relevant metadata such as headers,
// method, path, and IP address. This allows the `this.env` framework to standardize
// incoming requests across different protocols and frameworks, making it easier to implement
// environment recognition, routing, and analytics.
// This module is designed to be extensible, allowing for future ingress types to be added as
use actix_web::HttpRequest;
use crate::middleware::env_request::{EnvRequest, EnvRequestHttp, EnvRequestWs};
use std::collections::HashMap;
/// Parses an incoming Actix `HttpRequest` into an `EnvRequest`
/// recognized by `this.env`.
///
/// This function standardizes different request types (e.g., HTTP vs WebSocket)
/// into an internal `EnvRequest` enum used for environment analysis and routing.
///
/// # Arguments
///
/// * `req` - A reference to the incoming `HttpRequest`
///
/// # Returns
///
/// * `Option<EnvRequest>` - Returns an `EnvRequest` variant if parsing succeeds,
///                          or `None` if not recognized.
pub fn parse_env_request(req: &HttpRequest) -> Option<EnvRequest> {
    //log::debug!("this.env parser: entering parser fn");
    let mut headers = HashMap::new();
    for (key, value) in req.headers().iter() {
        if let Ok(val) = value.to_str() {
            headers.insert(key.to_string(), val.to_string());
        }
    }

    let host = req
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let ip = req.connection_info().realip_remote_addr().map(|s| s.to_string());
    let method = req.method().to_string();
    let path = req.path().to_string();
    let is_ws = req
        .headers()
        .get("upgrade")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false);

    if is_ws {
        log::debug!("this.env parser: websocket request parsed");
        Some(EnvRequest::Ws(EnvRequestWs {
            host,
            ip,
            headers,
            payload: None,
        }))
    } else {
        log::debug!("this.env parser: http request parsed");
        Some(EnvRequest::Http(EnvRequestHttp {
            host,
            ip,
            method,
            path,
            headers,
        }))
    }
}