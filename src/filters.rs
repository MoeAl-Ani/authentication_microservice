use std::borrow::Borrow;
use std::convert::TryFrom;
use std::pin::Pin;
use std::process;
use std::sync::{Mutex, MutexGuard, PoisonError, RwLock};
use std::task::{Context, Poll};
use std::thread::Thread;

use actix_service::{Service, Transform};
use actix_web::{dev::RequestHead, FromRequest, guard::Guard, http, HttpRequest, HttpResponse, web};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use actix_web::dev::{Payload, PayloadStream};
use actix_web::error::{ErrorUnauthorized, PayloadError};
use actix_web::http::{HeaderName, HeaderValue};
use actix_web::web::{Bytes, Data};
use futures::{Future, FutureExt, Stream, TryFutureExt, TryStreamExt};
use futures::future::{Either, err, ok, Ready};
use log::debug;

use crate::jwt_service::{JwtClaims, SessionType, verify};
use crate::UserPrinciple;

pub struct ContentTypeHeader;

pub struct MethodAllowed;

/// Guard filter for content type
impl Guard for ContentTypeHeader {
    fn check(&self, req: &RequestHead) -> bool {
        req.headers().contains_key(http::header::CONTENT_TYPE)
    }
}

/// Guard filter for allowed methods
impl Guard for MethodAllowed {
    fn check(&self, req: &RequestHead) -> bool {
        let inner = req.method.as_str();
        match inner {
            //"GET" => {true}
            "POST" => { true }
            "PATCH" => { true }
            "DELETE" => { true }
            "OPTION" => { true }
            _ => { false }
        }
    }
}

pub struct AuthFilter;

pub struct AuthFilterMiddleware<S> {
    service: S
}

impl<S, B> Transform<S> for AuthFilter
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
        B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthFilterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthFilterMiddleware { service })
    }
}

impl<S, B> Service for AuthFilterMiddleware<S>
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
        B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let path = req.path();
        if path.contains("/login/1") || path.contains("/login/2") {
            //Either::Left(self.service.call(req))
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            })
        } else {
            if req.headers().contains_key("Authorization") {
                let authorization = req.headers().get("Authorization").unwrap();
                let mut auth_value = authorization.to_str().unwrap();
                let mut jwt = "";
                for (index, v) in auth_value.char_indices() {
                    if index == 7 {
                        jwt = &auth_value[7..];
                        break;
                    }
                }

                match verify(&jwt.to_string()) {
                    None => {
                        Box::pin(async move {
                            let res = req.into_response(HttpResponse::Unauthorized().finish().into_body());
                            Ok(res)
                        })
                    }
                    Some(claim) => {
                        // found claim
                        let email = &claim.sub.unwrap();

                        let h = req.headers_mut();
                        h.insert(HeaderName::from_static("is_valid"), HeaderValue::try_from("true".to_string()).unwrap());
                        h.insert(HeaderName::from_static("email"), HeaderValue::try_from(email).unwrap());
                        h.insert(HeaderName::from_static("session_type"), HeaderValue::try_from(claim.session_type.unwrap().clone().to_string()).unwrap());
                        let fut = self.service.call(req);

                        Box::pin(async move {
                            let res = fut.await?;
                            Ok(res)
                        })
                    }
                }
            } else {
                debug!("no auth found");
                Box::pin(async move {
                    let res = req.into_response(HttpResponse::Unauthorized().finish().into_body());
                    Ok(res)
                })
            }
        }
    }
}

impl FromRequest for UserPrinciple {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload<PayloadStream>) -> Self::Future {
        if req.headers().contains_key("is_valid") {
            let email = req.headers().get("email").unwrap().to_str().unwrap();
            let session_type = req.headers().get("session_type").unwrap().to_str().unwrap().parse::<SessionType>().unwrap();
            ok(UserPrinciple {
                email: Some(email.to_string()),
                session_type: Some(session_type),
            })
        } else {
            err(ErrorUnauthorized("no valid session found"))
        }
    }
}


#[cfg(test)]
mod test {
    use std::ops::Add;
    use std::sync::Mutex;
    use std::thread;
    use std::time::SystemTime;

    use actix_web::{App, test, web};
    use actix_web::http::StatusCode;
    use chrono::{Duration, NaiveDateTime, Timelike, Utc};
    use env_logger::Env;
    use futures::task::SpawnExt;
    use tokio::task;

    use crate::{echo_resource, filters};
    use crate::jwt_service::{issue, SessionType};

    use super::*;

    #[actix_rt::test]
    async fn test_authorization_header_not_exist() {
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var("RUST_BACKTRACE", "1");
        env_logger::try_init();

        let c = web::Data::new(echo_resource::AppStateWithCounter::new());
        let mut app = test::init_service(App::new().wrap(filters::AuthFilter)
            .app_data(c.clone())
            .configure(echo_resource::config)).await;
        let req = test::TestRequest::with_header("content-type", "application/json").uri("/echo/counter").to_request();
        let resp = test::call_service(&mut app, req).await;
        println!("{}", resp.status());
        assert!(!resp.status().is_success());
        assert_eq!(StatusCode::UNAUTHORIZED, resp.status());

        // req add authorization header
        let mut req_builder = test::TestRequest::with_header("content-type", "application/json").uri("/echo/counter");
        req_builder = req_builder.header("Authorization", "crap");
        let resp = test::call_service(&mut app, req_builder.to_request()).await;
        println!("{}", resp.status());
        assert_eq!(StatusCode::UNAUTHORIZED, resp.status());

        // req add authorization with correct scheme but crap token
        let mut req_builder = test::TestRequest::with_header("content-type", "application/json").uri("/echo/counter");
        req_builder = req_builder.header("Authorization", "bearer crap");
        let resp = test::call_service(&mut app, req_builder.to_request()).await;
        println!("{}", resp.status());
        assert_eq!(StatusCode::UNAUTHORIZED, resp.status());

        let mut req_builder = test::TestRequest::with_header("content-type", "application/json").uri("/echo/counter");
        let valid_token = generate_valid_token("moe@gmail.com");
        req_builder = req_builder.header("Authorization", format!("bearer {}", valid_token));
        let resp = test::call_service(&mut app, req_builder.to_request()).await;
        println!("{}", resp.status());
        assert_eq!(StatusCode::OK, resp.status());
    }

    #[actix_rt::test]
    async fn test_user_principal_creations() {
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var("RUST_BACKTRACE", "1");
        env_logger::try_init();

        let c = web::Data::new(echo_resource::AppStateWithCounter::new());
        let mut app = test::init_service(App::new().wrap(filters::AuthFilter)
            .app_data(c.clone())
            .configure(echo_resource::config)).await;

        let mut req_builder = test::TestRequest::with_header("content-type", "application/json").uri("/echo/counter");
        let valid_token = generate_valid_token("moe@gmail");
        req_builder = req_builder.header("Authorization", format!("bearer {}", valid_token));
        let resp = test::call_service(&mut app, req_builder.to_request()).await;
        println!("{}", resp.status());
        assert_eq!(StatusCode::OK, resp.status());

        let mut req_builder = test::TestRequest::with_header("content-type", "application/json").uri("/echo/counter");
        let valid_token = generate_valid_token("ahmed@gmail");
        req_builder = req_builder.header("Authorization", format!("bearer {}", valid_token));
        let resp = test::call_service(&mut app, req_builder.to_request()).await;
        println!("{}", resp.status());
        assert_eq!(StatusCode::OK, resp.status());
    }

    fn generate_valid_token(email: &str) -> String {
        let mut claims = JwtClaims {
            aud: Some("".to_string()),
            exp: Utc::now().add(Duration::days(1)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            issuer: Some("infotamia".to_string()),
            jwt_id: Some("myid".to_string()),
            sub: Some(email.to_string()),
            access_token: Some("sometoken".to_string()),
            session_type: Some(SessionType::USER),
        };

        issue(&mut claims)
    }
}