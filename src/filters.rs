use std::pin::Pin;
use std::sync::{Mutex, MutexGuard, PoisonError, RwLock};
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{dev::RequestHead, guard::Guard, http, HttpRequest, HttpResponse, web};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use actix_web::dev::Payload;
use actix_web::error::PayloadError;
use actix_web::web::{Bytes, Data};
use futures::{Future, FutureExt, Stream, TryFutureExt, TryStreamExt};
use futures::future::{Either, ok, Ready};
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
            "POST" => {true}
            "PATCH" => {true}
            "DELETE" => {true}
            "OPTION" => {true}
            _ => {false}
        }
    }
}

pub struct AuthFilter;

pub struct AuthFilterMiddleware<S> {
    service: S
}

impl<S, B> Transform<S> for AuthFilter
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
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
        S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    //type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let path = req.path();
        if path.contains("/login/1") || path.contains("/login/2") {
            //Either::Left(self.service.call(req))
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;

                println!("Hi from response");
                Ok(res)
            })
        } else {
            if req.headers().contains_key("Authorization"){
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
                        //Either::Right(ok(req.into_response(HttpResponse::Unauthorized().finish().into_body())))
                        Box::pin(async move {
                            let res = req.into_response(HttpResponse::Unauthorized().finish().into_body());
                            Ok(res)
                        })
                    }
                    Some(claim) => {
                        let email = &claim.sub.unwrap();
                        let guard_user_principal = req.app_data::<web::Data<Mutex<UserPrinciple>>>().unwrap();
                        match guard_user_principal.lock() {
                            Ok(mut principal) => {
                                principal.email = Some(String::from(email.clone()));
                                principal.session_type = Some(claim.session_type.unwrap().clone());
                            }
                            Err(err) => {}
                        }
                        let fut = self.service.call(req);
                        //Either::Left(fut)

                        Box::pin(async move {
                            let res = fut.await?;
                            /// TODO clean up the principal or create new session clean filter
                            Ok(res)
                        })
                    }
                }
            } else {
                //Either::Right(ok(req.into_response(HttpResponse::Unauthorized().finish().into_body())))
                Box::pin(async move {
                    let res = req.into_response(HttpResponse::Unauthorized().finish().into_body());
                    Ok(res)
                })
            }
        }
    }
}


#[cfg(test)]
mod test {
    use std::ops::Add;
    use std::sync::Mutex;
    use std::time::SystemTime;

    use actix_web::{App, test, web};
    use actix_web::http::StatusCode;
    use chrono::{Duration, NaiveDateTime, Timelike, Utc};
    use env_logger::Env;

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
            .data(Mutex::new(UserPrinciple { email: None, session_type: None }))
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
        let valid_token = generate_valid_token();
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
            .data(Mutex::new(UserPrinciple { email: None, session_type: None }))
            .configure(echo_resource::config)).await;

        let mut req_builder = test::TestRequest::with_header("content-type", "application/json").uri("/echo/counter");
        let valid_token = generate_valid_token();
        req_builder = req_builder.header("Authorization", format!("bearer {}", valid_token));
        let resp = test::call_service(&mut app, req_builder.to_request()).await;
        println!("{}", resp.status());
        assert_eq!(StatusCode::OK, resp.status());

        let mut req_builder = test::TestRequest::with_header("content-type", "application/json").uri("/echo/counter");
        let valid_token = generate_valid_token();
        req_builder = req_builder.header("Authorization", format!("bearer {}", valid_token));
        let resp = test::call_service(&mut app, req_builder.to_request()).await;
        println!("{}", resp.status());
        assert_eq!(StatusCode::OK, resp.status());
    }

    fn generate_valid_token() -> String {
        let mut claims = JwtClaims {
            aud: Some("".to_string()),
            exp: Utc::now().add(Duration::days(1)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            issuer: Some("infotamia".to_string()),
            jwt_id: Some("myid".to_string()),
            sub: Some("moe@gmail.com".to_string()),
            access_token: Some("sometoken".to_string()),
            session_type: Some(SessionType::USER)
        };

        issue(&mut claims)
    }
}