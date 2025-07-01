// this.env/crate/src/middleware/actix.rs by suiGn
//! Actix adapter for `this.env`, responsible for converting incoming Actix requests
//! into a standardized `EnvRequest`.
use crate::middleware::env_request::{EnvRequest, EnvRequestHttp, EnvRequestWs};
use crate::env::{Env, EnvStatus};
use log::error;
use std::collections::HashMap;
use std::rc::Rc;
use std::task::{Context, Poll};
use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use futures_util::future::{ok, Ready, LocalBoxFuture};
use actix_web::{HttpResponse, http::StatusCode};
use actix_web::body::{BoxBody, EitherBody};
// embed templates
const HTML_PENDING: &str = include_str!("../html/pending_approval.html");
const HTML_BLOCKED: &str  = include_str!("../html/blocked.html");
/// ActixMiddleware is the bridge that transforms Actix requests into `EnvRequest`
/// instances and delegates handling to the shared handler logic.
pub struct ActixMiddleware;
impl<S, B> Transform<S, ServiceRequest> for ActixMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Transform = ActixMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ActixMiddlewareService {
            service: Rc::new(service),
        })
    }
}

/// The middleware service implementation that intercepts the request,
/// builds an `EnvRequest`, and invokes the handler pipeline.
pub struct ActixMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ActixMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        log::info!("🧩 ActixMiddleware: interceptando request {} {}", req.method(), req.path());
        let svc = Rc::clone(&self.service);
        // Build headers as HashMap<String, String>
        let mut headers = HashMap::new();
        for (key, value) in req.headers().iter() {
            if let Ok(val) = value.to_str() {
                headers.insert(key.to_string(), val.to_string());
            }
        }

        // Extract metadata
        let host = req
            .headers()
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let ip = req.connection_info().realip_remote_addr().map(|s| s.to_string());
        let method = req.method().to_string();
        let path = req.path().to_string();

        // Determine request type (WebSocket vs HTTP)
        let is_ws = req
            .headers()
            .get("upgrade")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("websocket"))
            .unwrap_or(false);
        let env_request_result = (|| {
            if is_ws {
                Some(EnvRequest::Ws(EnvRequestWs {
                    host,
                    ip,
                    headers,
                    payload: None,
                }))
            } else {
                Some(EnvRequest::Http(EnvRequestHttp {
                    host,
                    ip,
                    method,
                    path,
                    headers,
                }))
            }
        })();

        // Determine if client expects HTML or JSON/text
        let accepts_html = req
            .headers()
            .get("accept")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("text/html"))
            .unwrap_or(false);

Box::pin(async move {
    if let Some(env_request) = env_request_result {
        match Env::resolve(&env_request) {

/* ▗▄▖ ▗▄▄▖ ▗▄▄▖ ▗▄▄▖  ▗▄▖ ▗▖  ▗▖▗▄▄▄▖▗▄▄▄ 
  ▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌  ▐▌▐▌   ▐▌  █
  ▐▛▀▜▌▐▛▀▘ ▐▛▀▘ ▐▛▀▚▖▐▌ ▐▌▐▌  ▐▌▐▛▀▀▘▐▌  █
  ▐▌ ▐▌▐▌   ▐▌   ▐▌ ▐▌▝▚▄▞▘ ▝▚▞▘ ▐▙▄▄▖▐▙▄▄*/
            Ok(EnvStatus::Approved) => {
                // ✅  Approved
                return svc.call(req).await.map(ServiceResponse::map_into_left_body);
            }
/*▗▄▄▖ ▗▄▄▄▖▗▖  ▗▖▗▄▄▄ ▗▄▄▄▖▗▖  ▗▖ ▗▄▄▖
  ▐▌ ▐▌▐▌   ▐▛▚▖▐▌▐▌  █  █  ▐▛▚▖▐▌▐▌   
  ▐▛▀▘ ▐▛▀▀▘▐▌ ▝▜▌▐▌  █  █  ▐▌ ▝▜▌▐▌▝▜▌
  ▐▌   ▐▙▄▄▖▐▌  ▐▌▐▙▄▄▀▗▄█▄▖▐▌  ▐▌▝▚▄▞▘*/
            Ok(EnvStatus::PendingApproval(_)) => {
                // ⏳ Pendiending approval
                if accepts_html {
                    let resp = HttpResponse::build(StatusCode::UNAUTHORIZED)
                        .content_type("text/html")
                        .body(HTML_PENDING);
                    return Ok(req.into_response(resp.map_into_right_body()));
                } else {
                    let resp = HttpResponse::build(StatusCode::UNAUTHORIZED)
                        .content_type("text/plain")
                        .body("PendingApproval");
                    return Ok(req.into_response(resp.map_into_right_body()));
                }
            }
/*▗▄▄▖ ▗▖    ▗▄▖  ▗▄▄▖▗▖ ▗▖▗▄▄▄▖▗▄▄▄ 
  ▐▌ ▐▌▐▌   ▐▌ ▐▌▐▌   ▐▌▗▞▘▐▌   ▐▌  █
  ▐▛▀▚▖▐▌   ▐▌ ▐▌▐▌   ▐▛▚▖ ▐▛▀▀▘▐▌  █
  ▐▙▄▞▘▐▙▄▄▖▝▚▄▞▘▝▚▄▄▖▐▌ ▐▌▐▙▄▄▖▐▙▄▄▀*/ 
            Ok(EnvStatus::Blocked(_)) => {
                // ⛔ Blocked 
                if accepts_html {
                    let resp = HttpResponse::build(StatusCode::FORBIDDEN)
                        .content_type("text/html")
                        .body(HTML_BLOCKED);
                    return Ok(req.into_response(resp.map_into_right_body()));
                } else {
                    let resp = HttpResponse::build(StatusCode::FORBIDDEN)
                        .content_type("text/plain")
                        .body("Blocked");
                    return Ok(req.into_response(resp.map_into_right_body()));
                }
            }
/*▗▄▄▄▖▗▄▄▖ ▗▄▄▖  ▗▄▖ ▗▄▄▖ 
  ▐▌   ▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌
  ▐▛▀▀▘▐▛▀▚▖▐▛▀▚▖▐▌ ▐▌▐▛▀▚▖
  ▐▙▄▄▖▐▌ ▐▌▐▌ ▐▌▝▚▄▞▘▐▌ ▐▌*/                        
            Err(msg) => {
                error!("this.env error: {:?}", msg);
                let resp = HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body("Internal this.env error");
                return Ok(req.into_response(resp.map_into_right_body()));
            }
        }
    }

    // if we couldn't resolve the environment, just pass through
    svc.call(req).await.map(ServiceResponse::map_into_left_body)
})
    }
}