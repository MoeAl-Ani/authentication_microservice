use std::pin::Pin;

use actix_service::{Service, Transform};
use actix_web::{Error, HttpResponse};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::{HeaderMap, HeaderName, HeaderValue};
use futures::future::{ok, Ready};
use futures::Future;
use futures::task::Context;
use tokio::macros::support::Poll;

pub struct CorsFilter;

pub struct CorsFilterMiddleware<S> {
    service: S
}

impl<S, B> Transform<S> for CorsFilter
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
        B: 'static, {
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CorsFilterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CorsFilterMiddleware { service })
    }
}

impl<S, B> Service for CorsFilterMiddleware<S>
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
        B: 'static, {
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        /// check http verb is it options or not
        if req.method().as_str() == "OPTIONS" {
            Box::pin(async move {
                let mut res = req.into_response(HttpResponse::Ok().finish().into_body());
                let headers = res.headers_mut();
                add_non_options_headers(headers);
                add_options_headers(headers);
                Ok(res)
            })
        } else {
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            })
        }
    }
}

fn add_options_headers(headers: &mut HeaderMap) {
    headers.insert(HeaderName::from_static("access-control-allow-headers"), HeaderValue::from_static("Content-Type, Authorization, X-Requested-With"));
    headers.insert(HeaderName::from_static("access-control-allow-methods"), HeaderValue::from_static("OPTIONS, GET, POST, PATCH, DELETE"));
    headers.insert(HeaderName::from_static("access-control-expose-headers"), HeaderValue::from_static("Location"));
}

fn add_non_options_headers(headers: &mut HeaderMap) {
    headers.insert(HeaderName::from_static("access-control-allow-credentials"), HeaderValue::from_static("true"));
    headers.insert(HeaderName::from_static("access-control-allow-origin"), HeaderValue::from_static("*"));
    headers.insert(HeaderName::from_static("access-control-allow-methods"), HeaderValue::from_static("OPTIONS, GET, POST, PATCH, DELETE"));
    headers.insert(HeaderName::from_static("access-control-expose-headers"), HeaderValue::from_static("Location"));
}

