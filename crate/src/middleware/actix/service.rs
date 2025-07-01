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
        use crate::middleware::env_request::{EnvRequest, EnvRequestHttp, EnvRequestWs};
        use crate::env::{Env, EnvStatus};
        use actix_web::{HttpResponse, http::StatusCode};

        log::info!("🧩 ActixMiddleware: interceptando request {} {}", req.method(), req.path());

        let svc = Rc::clone(&self.service);
        let config = self.config.clone();

        let mut headers = std::collections::HashMap::new();
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

        let accepts_html = req
            .headers()
            .get("accept")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("text/html"))
            .unwrap_or(false);

        if config.manual_mode {
            if let Some(env_request) = &env_request_result {
                match Env::resolve(env_request) {
                    Ok(status) => log::info!("this.env status (manual mode): {:?}", status),
                    Err(e) => log::error!("this.env resolve error (manual mode): {:?}", e),
                }
            }
            return Box::pin(async move {
                let res = svc.call(req).await?;
                Ok(res.map_into_left_body())
            });
        }

        let decision_status = match env_request_result.as_ref().and_then(|env_request| Env::resolve(env_request).ok()) {
            Some(EnvStatus::PendingApproval(_)) if config.allow_pending => EnvStatus::Approved,
            Some(EnvStatus::Blocked(_)) if config.allow_blocked => EnvStatus::Approved,
            Some(status) => status,
            None => {
                if let Some(env_request) = &env_request_result {
                    if let Err(e) = Env::resolve(env_request) {
                        log::error!("this.env resolve error: {e:?}");
                    }
                }
                EnvStatus::Blocked("internal-error".into())
            }
        };

        let svc_clone = Rc::clone(&svc);
        let req_clone = req;

        Box::pin(async move {
            if let Some(_env_request) = env_request_result {
                match decision_status {
/* ▗▄▖ ▗▄▄▖ ▗▄▄▖ ▗▄▄▖  ▗▄▖ ▗▖  ▗▖▗▄▄▄▖▗▄▄▄ 
  ▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌  ▐▌▐▌   ▐▌  █
  ▐▛▀▜▌▐▛▀▘ ▐▛▀▘ ▐▛▀▚▖▐▌ ▐▌▐▌  ▐▌▐▛▀▀▘▐▌  █
  ▐▌ ▐▌▐▌   ▐▌   ▐▌ ▐▌▝▚▄▞▘ ▝▚▞▘ ▐▙▄▄▖▐▙▄▄*/
                    EnvStatus::Approved => {
                        let res = svc_clone.call(req_clone).await?;
                        return Ok(res.map_into_left_body());
                    }
/*▗▄▄▖ ▗▄▄▄▖▗▖  ▗▖▗▄▄▄ ▗▄▄▄▖▗▖  ▗▖ ▗▄▄▖
  ▐▌ ▐▌▐▌   ▐▛▚▖▐▌▐▌  █  █  ▐▛▚▖▐▌▐▌   
  ▐▛▀▘ ▐▛▀▀▘▐▌ ▝▜▌▐▌  █  █  ▐▌ ▝▜▌▐▌▝▜▌
  ▐▌   ▐▙▄▄▖▐▌  ▐▌▐▙▄▄▀▗▄█▄▖▐▌  ▐▌▝▚▄▞▘*/
                    EnvStatus::PendingApproval(_) => {
                        let wants_html = accepts_html || config.prefer_html;
                        if wants_html {
                            let resp = HttpResponse::build(StatusCode::UNAUTHORIZED)
                                .content_type("text/html")
                                .body(include_str!("../../html/pending_approval.html"));
                            return Ok(req_clone.into_response(resp.map_into_right_body()));
                        } else {
                            let resp = HttpResponse::build(StatusCode::UNAUTHORIZED)
                                .content_type("text/plain")
                                .body("PendingApproval");
                            return Ok(req_clone.into_response(resp.map_into_right_body()));
                        }
                    }
/*▗▄▄▖ ▗▖    ▗▄▖  ▗▄▄▖▗▖ ▗▖▗▄▄▄▖▗▄▄▄ 
  ▐▌ ▐▌▐▌   ▐▌ ▐▌▐▌   ▐▌▗▞▘▐▌   ▐▌  █
  ▐▛▀▚▖▐▌   ▐▌ ▐▌▐▌   ▐▛▚▖ ▐▛▀▀▘▐▌  █
  ▐▙▄▞▘▐▙▄▄▖▝▚▄▞▘▝▚▄▄▖▐▌ ▐▌▐▙▄▄▖▐▙▄▄▀*/ 
                    EnvStatus::Blocked(_) => {
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