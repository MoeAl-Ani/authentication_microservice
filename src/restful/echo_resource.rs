use std::sync::{Mutex, RwLock};

use actix_web::{App, Error, get, guard, HttpRequest, HttpResponse, HttpServer, post, Responder, web};
use actix_web::body::Body;
use actix_web::web::Data;
use futures::future::{ready, Ready};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{main, UserPrinciple};
use crate::services::jwt_service::SessionType;

use crate::exceptions::error_base::{ErrorResponse, HttpErrorCode};
use crate::filters::authentication_filter::{ContentTypeHeader, MethodAllowed};
use uuid::Uuid;
use sqlx::{Pool, MySql, MySqlPool, Executor};
use sqlx::mysql::MySqlDone;
use crate::entities::user_entity::UserEntity;

pub struct AppStateWithCounter {
    pub counter: Mutex<i32>,
}

impl AppStateWithCounter {
    pub fn new() -> Self {
        AppStateWithCounter {
            counter: Mutex::new(0)
        }
    }
}

#[derive(Serialize)]
struct EchoModel {
    name: &'static str
}

#[derive(Deserialize)]
struct Info {
    echo_id: u32,
    friend: String,
}

impl Responder for EchoModel {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

/// general echo resource
#[get("/")]
pub async fn echo() -> impl Responder {
    HttpResponse::Ok().body("Hello from server!")
}

/// shared state counter example
#[get("/counter")]
pub async fn counter(data: web::Data<AppStateWithCounter>, principal: Option<UserPrinciple>) -> impl Responder {
    match data.counter.lock() {
        Ok(mut c) => {
            *c += 1;
            HttpResponse::Ok().body(format!("counter = {}", c))
        }
        Err(_) => {
            HttpResponse::Ok().body(format!("counter not incremented"))
        }
    }
}

/// model responder serialization
#[get("/model")]
pub async fn users() -> impl Responder {
    EchoModel { name: "moe" }
}

/// path segment example
#[get("/model/{echo_id}/{friend}")]
async fn index(info: web::Path<Info>) -> Result<String, Error> {
    Ok(format!("Welcome {}, echo_id {}!", info.friend, info.echo_id))
}

#[get("/error")]
async fn error() -> Result<String, HttpErrorCode> {
    Err(HttpErrorCode::BadRequest { message: ErrorResponse { message: "missing user id".into(), error_code: "MissingUserId".into() } })
}

#[post("/write_user")]
pub async fn mock(user: UserPrinciple, pool: web::Data<MySqlPool>) -> impl Responder {
    // let mut conn = pool.get_conn().unwrap().unwrap();
    // let statement = conn.prep(r"INSERT INTO user(first_name, last_name, email, phone_number, language_id) VALUES(:first_name,:last_name,:email,:phone_number,:language_id)").unwrap();
    // let mut tx = conn.start_transaction(TxOpts::default()).unwrap();
    let e = UserEntity {
        email: format!("mock@{}", Uuid::new_v4().to_string().get(0..10).unwrap()),
        phone_number: "0403231145".to_string(),
        language_id: 1,
        first_name: None,
        last_name: None,
        id: None,
        salt: None,
        verifier: None
    };
    let done: Result<MySqlDone, sqlx::Error> = sqlx::query("INSERT INTO user(first_name, last_name, email, phone_number, language_id) VALUES(?,?,?,?,?)")
        .bind(&e.first_name)
        .bind(&e.last_name)
        .bind(&e.email)
        .bind(&e.phone_number)
        .bind(e.language_id).execute(pool.get_ref()).await;
    Some(e)
}


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/echo")
        .service(echo)
        .service(counter)
        .service(users)
        .service(index)
        .service(error)
        .service(mock));
}

#[cfg(test)]
mod tests {
    use actix_web::{App, test, web};
    use actix_web::http::StatusCode;

    use crate::restful::echo_resource;

    use super::*;

    #[actix_rt::test]
    async fn test_error() {
        let mut app = test::init_service(App::new().configure(echo_resource::config)).await;
        let req = test::TestRequest::with_header("content-type", "application/json").uri("/echo/error").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(!resp.status().is_success());
        assert_eq!(StatusCode::BAD_REQUEST, resp.status())
    }

    #[actix_rt::test]
    async fn test_counter() {
        let c = web::Data::new(echo_resource::AppStateWithCounter::new());
        let mut app = test::init_service(App::new().app_data(c.clone()).configure(echo_resource::config)).await;
        let req = test::TestRequest::with_uri("/echo/counter").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
        assert_eq!(StatusCode::OK, resp.status());
    }
}