//! Actix middleware definition
//!
//! This module defines the `ActixMiddleware` struct which implements
//! the `Transform` trait to intercept and process requests in Actix.
use std::rc::Rc;
use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use actix_web::body::{EitherBody, BoxBody};
use futures_util::future::{ok, Ready};
use crate::middleware::actix::ActixMwConfig;
use super::service::ActixMiddlewareService;
/// The `ActixMiddleware` struct is the entry point for Actix integration with `this.env`.
/// It wraps Actix services and prepares the `Transform`.
#[derive(Clone)]
pub struct ActixMiddleware {
    pub(crate) config: ActixMwConfig,
}

impl ActixMiddleware {
    /// Create a new `ActixMiddleware` with a given configuration.
    pub fn new(config: ActixMwConfig) -> Self {
        Self { config }
    }

    /// Create an `ActixMiddleware` from a given configuration.
    pub fn config(cfg: ActixMwConfig) -> Self {
        Self { config: cfg }
    }
}

impl Default for ActixMiddleware {
    fn default() -> Self {
        Self {
            config: ActixMwConfig::default(),
        }
    }
}

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
            config: self.config.clone(),
        })
    }
}