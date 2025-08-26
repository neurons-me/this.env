// This file is part of the this.env crate.
// It provides middleware for Actix Web to handle internal routes and request logging.
// by suiGn
use actix_web::HttpMessage;
use actix_web::{dev::ServiceRequest, HttpResponse};
use actix_files::NamedFile;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use crate::env::methods::get_requests_logs::get_request_logs;
use serde_json::json;
/// Check if the request is for an internal this.env route and restrict access to localhost
pub fn intercept_internal_routes(req: &ServiceRequest) -> Option<HttpResponse> {
    let path = req.path();

    if path.starts_with("/__env/") {
        let remote_addr = req.peer_addr().map(|addr| addr.ip());

        let is_localhost = match remote_addr {
            Some(IpAddr::V4(ip)) => ip.octets() == [127, 0, 0, 1],
            Some(IpAddr::V6(ip)) => ip.octets() == [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            _ => false,
        };

        if !is_localhost {
            return Some(HttpResponse::Forbidden()
                .body("Access to internal __env routes is restricted to localhost."));
        }

        return match path {
            "/__env/status" => Some(HttpResponse::Ok().json(json!({
                "status": "ok",
                "env": "this.env"
            }))),
            "/__env/ui" => Some(HttpResponse::Ok().content_type("text/html").body(r#"
                <html><body>
                    <h1>this.env</h1>
                    <p>Status: OK</p>
                </body></html>
            "#)),
            "/__env/logs" => {
                let file_path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("src/html/requests_history_logs.html");
                match NamedFile::open(file_path) {
                    Ok(file) => Some(file.into_response(req.request())),
                    Err(_) => Some(HttpResponse::InternalServerError().body("Could not load logs HTML")),
                }
            }
            "/__env/" => {
                let file_path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("src/html/env.html");
                match NamedFile::open(file_path) {
                    Ok(file) => Some(file.into_response(req.request())),
                    Err(_) => Some(HttpResponse::InternalServerError().body("Could not load env HTML")),
                }
            }
            "/__env/log-history" => {
                match req.extensions().get::<std::sync::Arc<rusqlite::Connection>>() {
                    Some(conn) => {
                        use std::collections::HashMap;
                        let query_string = req.query_string();
                        let query_params: HashMap<String, String> = serde_urlencoded::from_str(query_string).unwrap_or_default();
                        let limit = query_params.get("limit").and_then(|v| v.parse().ok());
                        let offset = query_params.get("offset").and_then(|v| v.parse().ok());

                        match get_request_logs(conn, None, limit, offset) {
                            Ok(logs) => Some(HttpResponse::Ok().json(logs)),
                            Err(err) => Some(HttpResponse::InternalServerError().json(json!({
                                "error": "Failed to retrieve logs",
                                "details": err.to_string()
                            }))),
                        }
                    },
                    None => {
                        log::error!("__env/log-history: No DB connection found in extensions");
                        Some(HttpResponse::InternalServerError().json(json!({
                            "error": "No DB connection found"
                        })))
                    }
                }
            }
            _ => Some(HttpResponse::NotFound().body("Unknown __env route")),
        };
    }
    None
}