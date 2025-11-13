//  this.env/crate/src/middleware/env_request.rs by suiGn
//  Module for standardizing inbound request translation to internal EnvRequest enums.
//  env_request.rs only declares the portable structs/enums that
//  describe an incoming request in a framework-neutral way.
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
/*▗▄▄▄▖▗▖  ▗▖▗▄▄▖ ▗▄▄▄▖ ▗▄▄▖
    █   ▝▚▞▘ ▐▌ ▐▌▐▌   ▐▌   
    █    ▐▌  ▐▛▀▘ ▐▛▀▀▘ ▝▀▚▖
    █    ▐▌  ▐▌   ▐▙▄▄▖▗▄▄▞▘
This module defines the `EnvRequest` enum, which encapsulates different types of requests. */
/// A request descriptor representing the source of a request.
/// This enum abstracts various ingress types (e.g., HTTP, WebSocket, CLI)
/// into a unified request structure for environment recognition and handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvRequest {
    /// HTTP-based request (e.g., Actix, Express, Hyper, etc.)
    Http(EnvRequestHttp),
    /// WebSocket-based request
    Ws(EnvRequestWs),
    /// Command-line or programmatic trigger
    Cli(EnvRequestCli),
}

/// A simplified, serializable form of EnvRequest for response bodies or logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvRequestInfo {
    pub r#type: String,
    pub host: String,
    pub ip: Option<String>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub args: Option<Vec<String>>,
}

impl From<&EnvRequest> for EnvRequestInfo {
    fn from(req: &EnvRequest) -> Self {
        match req {
            EnvRequest::Http(http) => EnvRequestInfo {
                r#type: "http".into(),
                host: http.host.clone(),
                ip: http.ip.clone(),
                method: Some(http.method.clone()),
                path: Some(http.path.clone()),
                headers: Some(http.headers.clone()),
                args: None,
            },
            EnvRequest::Ws(ws) => EnvRequestInfo {
                r#type: "ws".into(),
                host: ws.host.clone(),
                ip: ws.ip.clone(),
                method: None,
                path: None,
                headers: Some(ws.headers.clone()),
                args: None,
            },
            EnvRequest::Cli(cli) => EnvRequestInfo {
                r#type: "cli".into(),
                host: "localhost".into(),
                ip: None,
                method: Some(cli.command.clone()),
                path: Some(cli.args.join(" ")),
                headers: None,
                args: Some(cli.args.clone()),
            },
        }
    }
}
/*▗▄▄▖▗▄▄▄▖▗▄▄▖ ▗▖ ▗▖ ▗▄▄▖▗▄▄▄▖▗▖ ▗▖▗▄▄▖ ▗▄▄▄▖ ▗▄▄▖
 ▐▌     █  ▐▌ ▐▌▐▌ ▐▌▐▌     █  ▐▌ ▐▌▐▌ ▐▌▐▌   ▐▌   
  ▝▀▚▖  █  ▐▛▀▚▖▐▌ ▐▌▐▌     █  ▐▌ ▐▌▐▛▀▚▖▐▛▀▀▘ ▝▀▚▖
 ▗▄▄▞▘  █  ▐▌ ▐▌▝▚▄▞▘▝▚▄▄▖  █  ▝▚▄▞▘▐▌ ▐▌▐▙▄▄▖▗▄▄▞▘*/
///this.env - Environment Request Types
///This module defines the request types used by this.env to handle various ingress sources.
///It includes HTTP, WebSocket, and CLI requests, each with its own structure.
///These types are used to standardize how requests are processed and analyzed within the this.env framework.
///This allows for consistent handling of requests across different protocols and frameworks,
///making it easier to implement environment recognition, routing, and analytics.
///This module is designed to be extensible, allowing for future ingress types to be added as needed. */
/// A request made through HTTP, carrying host, path, headers and metadata.
/* 
▗▖ ▗▖▗▄▄▄▖▗▄▄▄▖▗▄▄▖ 
▐▌ ▐▌  █    █  ▐▌ ▐▌
▐▛▀▜▌  █    █  ▐▛▀▘ 
▐▌ ▐▌  █    █  ▐▌ */   
/// Represents an HTTP request with metadata and headers.                 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvRequestHttp {
    /// Host header or domain of the incoming request.
    pub host: String,
    /// Optional remote IP address, if available.
    pub ip: Option<String>,
    /// HTTP method (e.g., GET, POST).
    pub method: String,
    /// Request path (e.g., /api/data).
    pub path: String,
    /// Headers from the incoming request, flattened as string pairs.
    pub headers: HashMap<String, String>,
}
/*
▗▖ ▗▖▗▄▄▄▖▗▄▄▖  ▗▄▄▖ ▗▄▖  ▗▄▄▖▗▖ ▗▖▗▄▄▄▖▗▄▄▄▖
▐▌ ▐▌▐▌   ▐▌ ▐▌▐▌   ▐▌ ▐▌▐▌   ▐▌▗▞▘▐▌     █  
▐▌ ▐▌▐▛▀▀▘▐▛▀▚▖ ▝▀▚▖▐▌ ▐▌▐▌   ▐▛▚▖ ▐▛▀▀▘  █  
▐▙█▟▌▐▙▄▄▖▐▙▄▞▘▗▄▄▞▘▝▚▄▞▘▝▚▄▄▖▐▌ ▐▌▐▙▄▄▖  █  */     
/// A WebSocket-based request, including connection metadata and payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvRequestWs {
    /// Host header or domain from the websocket upgrade request.
    pub host: String,
    /// Optional IP address of the connecting client.
    pub ip: Option<String>,
    /// Flattened headers at the moment of upgrade.
    pub headers: HashMap<String, String>,
    /// Optional string-based payload received.
    pub payload: Option<String>,
}
/*
 ▗▄▄▖▗▖   ▗▄▄▄▖
▐▌   ▐▌     █  
▐▌   ▐▌     █  
▝▚▄▄▖▐▙▄▄▖▗▄█▄▖*/
/// A CLI-triggered request, representing non-network system events or invocations.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvRequestCli {
    /// The command invoked from CLI or automation.
    pub command: String,
    /// Arguments passed to the command.
    pub args: Vec<String>,
    /// Environment variables or runtime context.
    pub env: HashMap<String, String>,
}
