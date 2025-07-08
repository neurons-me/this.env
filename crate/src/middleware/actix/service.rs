//this.env/crate/src/middleware/actix/service.rs
// by suiGn
use std::rc::Rc;
use std::task::{Context, Poll};
use actix_service::Service;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use actix_web::body::{BoxBody, EitherBody};
use futures_util::future::LocalBoxFuture;
use crate::middleware::actix::ActixMwConfig;
use serde_json::json;
use crate::middleware::actix::env_request_parser::parse_env_request;
use crate::middleware::env_request::EnvRequest;
use rusqlite::Connection;
use actix_web::{HttpResponse, http::StatusCode};
/// The middleware service implementation that intercepts the request,
/// builds an `EnvRequest`, and invokes the handler pipeline.
pub struct ActixMiddlewareService<S> {
    pub(crate) service: Rc<S>,
    pub(crate) config: ActixMwConfig,
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
        use crate::env::{Env, EnvStatus};
        let port = req.connection_info().host().split(':').nth(1).unwrap_or("unknown").to_string();
        if self.config.manual_mode {
            log::info!("this.env [{}] ActixMiddleware (manual): {} {}", port, req.method(), req.path());
        } else {
            log::info!("this.env [{}] ActixMiddleware: {} {}", port, req.method(), req.path());
        }

        let conn = match Connection::open(".db") {
            Ok(c) => c,
            Err(e) => {
                log::error!("this.env middleware: failed to open database: {:?}", e);
                return Box::pin(async {
                    let resp = HttpResponse::InternalServerError().finish();
                    Ok(req.into_response(resp.map_into_right_body()))
                });
            }
        };

        let svc = Rc::clone(&self.service);
        let config = self.config.clone();
        let env_request_result = parse_env_request(req.request());
        let accepts_html = req
            .headers()
            .get("Accept")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.contains("text/html"))
            .unwrap_or(false);

        if config.manual_mode {
            if let Some(env_request) = &env_request_result {
                match Env::resolve(env_request, &conn) {
                    status => log::info!("this.env status (manual mode): {:?}", status),
                }
            }
            return Box::pin(async move {
                let res = svc.call(req).await?;
                Ok(res.map_into_left_body())
            });
        }

        let decision_status = match &env_request_result {
            Some(env_request) => {
                if let Some(EnvRequest::Http(http)) = &env_request_result {
                    log::debug!("this.env request: [{}] {} {}", http.host, http.method, http.path);
                }
                let status = Env::resolve(env_request, &conn);
                match status {
                    EnvStatus::PendingApproval { env_request, reason: _ } if config.allow_pending => {
                        EnvStatus::Approved { env_request: env_request.clone() }
                    }
                    EnvStatus::Blocked { env_request, reason: _ } if config.allow_blocked => {
                        EnvStatus::Approved { env_request: env_request.clone() }
                    }
                    other => other,
                }
            }
            None => {
                // Fall‑back: create a dummy CLI request and mark as blocked
                EnvStatus::Blocked {
                    env_request: crate::middleware::env_request::EnvRequestInfo::from(&EnvRequest::Cli(Default::default())),
                    reason: "internal-error".into(),
                }
            }
        };

    log::info!("this.env decision: {:?}", match &decision_status {
        EnvStatus::Approved { .. } => "Approved",
        EnvStatus::PendingApproval { .. } => "PendingApproval",
        EnvStatus::Blocked { .. } => "Blocked",
    });
        let svc_clone = Rc::clone(&svc);
        let req_clone = req;
        // let req = &env_req.req; // (line to delete)
        Box::pin(async move {
            if let Some(_env_request) = env_request_result {
                match decision_status {
/* ▗▄▖ ▗▄▄▖ ▗▄▄▖ ▗▄▄▖  ▗▄▖ ▗▖  ▗▖▗▄▄▄▖▗▄▄▄ 
  ▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌  ▐▌▐▌   ▐▌  █
  ▐▛▀▜▌▐▛▀▘ ▐▛▀▘ ▐▛▀▚▖▐▌ ▐▌▐▌  ▐▌▐▛▀▀▘▐▌  █
  ▐▌ ▐▌▐▌   ▐▌   ▐▌ ▐▌▝▚▄▞▘ ▝▚▞▘ ▐▙▄▄▖▐▙▄▄*/
                    EnvStatus::Approved { env_request: _ } => {
                        let res = svc_clone.call(req_clone).await?;
                        return Ok(res.map_into_left_body());
                    }
/*▗▄▄▖ ▗▄▄▄▖▗▖  ▗▖▗▄▄▄ ▗▄▄▄▖▗▖  ▗▖ ▗▄▄▖
  ▐▌ ▐▌▐▌   ▐▛▚▖▐▌▐▌  █  █  ▐▛▚▖▐▌▐▌   
  ▐▛▀▘ ▐▛▀▀▘▐▌ ▝▜▌▐▌  █  █  ▐▌ ▝▜▌▐▌▝▜▌
  ▐▌   ▐▙▄▄▖▐▌  ▐▌▐▙▄▄▀▗▄█▄▖▐▌  ▐▌▝▚▄▞▘*/
                    EnvStatus::PendingApproval { env_request: env_req, reason: _ } => {
                        // Use EnvRequestInfo::from(env_req) for serialization.
                        let info = crate::middleware::env_request::EnvRequestInfo::from(env_req);
                        let json_body = json!({
                            "status": "pending",
                            "message": "Awaiting user approval",
                            "env_request": info
                        }).to_string();
                        let resp = HttpResponse::build(StatusCode::UNAUTHORIZED)
                            .content_type("application/json")
                            .body(json_body);
                        return Ok(req_clone.into_response(resp.map_into_right_body()));
                    }
/*▗▄▄▖ ▗▖    ▗▄▖  ▗▄▄▖▗▖ ▗▖▗▄▄▄▖▗▄▄▄ 
  ▐▌ ▐▌▐▌   ▐▌ ▐▌▐▌   ▐▌▗▞▘▐▌   ▐▌  █
  ▐▛▀▚▖▐▌   ▐▌ ▐▌▐▌   ▐▛▚▖ ▐▛▀▀▘▐▌  █
  ▐▙▄▞▘▐▙▄▄▖▝▚▄▞▘▝▚▄▄▖▐▌ ▐▌▐▙▄▄▖▐▙▄▄▀*/ 
                    EnvStatus::Blocked { env_request: _, reason: _ } => {
                        let wants_html = accepts_html || config.prefer_html;
                        if wants_html {
                            let resp = HttpResponse::build(StatusCode::FORBIDDEN)
                                .content_type("text/html")
                                .body(include_str!("../../html/blocked.html"));
                            return Ok(req_clone.into_response(resp.map_into_right_body()));
                        } else {
                            let resp = HttpResponse::build(StatusCode::FORBIDDEN)
                                .content_type("text/plain")
                                .body("Blocked");
                            return Ok(req_clone.into_response(resp.map_into_right_body()));
                        }
                    }
                }
            }
            let res = svc_clone.call(req_clone).await?;
            Ok(res.map_into_left_body())
        })
    }
}