//! # Actix Web Middleware
use crate::{api::ApiError, util::*, DriverError, HEADER_AUTHORISATION_NAME};
use actix_identity::{IdentityPolicy, IdentityService};
use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error, Result as ActixWebResult,
};
use futures::{
    future::{ok, FutureResult},
    Future, Poll,
};
use prometheus::{HistogramTimer, HistogramVec, IntCounterVec};
use std::fmt;

/// Header identity policy middleware.
#[derive(Debug)]
pub struct HeaderIdentityPolicy {
    header: String,
}

impl HeaderIdentityPolicy {
    /// Create new identity service.
    pub fn identity_service() -> IdentityService<Self> {
        IdentityService::new(HeaderIdentityPolicy::default())
    }
}

impl Default for HeaderIdentityPolicy {
    fn default() -> Self {
        Self {
            header: HEADER_AUTHORISATION_NAME.to_owned(),
        }
    }
}

impl IdentityPolicy for HeaderIdentityPolicy {
    type Future = ActixWebResult<Option<String>, Error>;
    type ResponseFuture = ActixWebResult<(), Error>;

    fn from_request(&self, request: &mut ServiceRequest) -> Self::Future {
        let service_key = match request.headers().get(&self.header) {
            Some(value) => {
                let value = value
                    .to_str()
                    .map_err(|_err| ApiError::Unauthorised(DriverError::HttpHeader))?;
                HeaderAuth::parse_key(value)
            }
            None => None,
        };
        Ok(service_key)
    }

    fn to_response<B>(
        &self,
        _id: Option<String>,
        _changed: bool,
        _response: &mut ServiceResponse<B>,
    ) -> Self::ResponseFuture {
        Ok(())
    }
}

/// Metrics middleware constructor.
pub struct Metrics {
    count: IntCounterVec,
    latency: HistogramVec,
}

impl fmt::Debug for Metrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Metrics {{ count, latency }}")
    }
}

impl Metrics {
    pub fn new(count: IntCounterVec, latency: HistogramVec) -> Self {
        Self { count, latency }
    }
}

impl<S, B> Transform<S> for Metrics
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MetricsMiddleware {
            service,
            count: self.count.clone(),
            latency: self.latency.clone(),
        })
    }
}

/// Metrics middleware.
pub struct MetricsMiddleware<S> {
    service: S,
    count: IntCounterVec,
    latency: HistogramVec,
}

impl<S> fmt::Debug for MetricsMiddleware<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MetricsMiddleware {{ service, count, latency }}")
    }
}

impl<S, B> Service for MetricsMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        // TODO(fix): Add path as label value (&[req.path()]).
        // <https://github.com/actix/actix-web/issues/833>
        let timer = self.latency.with_label_values(&["/"]).start_timer();
        let timer = ok::<HistogramTimer, Self::Error>(timer);
        let count = self.count.clone();

        Box::new(
            self.service
                .call(req)
                .join(timer)
                .and_then(move |(res, timer)| {
                    timer.observe_duration();
                    count
                        .with_label_values(&["/", res.status().as_str()])
                        .inc_by(1);
                    Ok(res)
                }),
        )
    }
}
