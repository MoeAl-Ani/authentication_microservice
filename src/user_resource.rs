use std::sync::{Mutex, RwLock};

use actix_web::{App, Error, get, guard, HttpRequest, HttpResponse, HttpServer, post, Responder, web};
use actix_web::body::Body;
use actix_web::web::Data;
use futures::future::{ready, Ready};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{main, UserPrinciple, user_dao};
use crate::jwt_service::SessionType;

use super::error_base::{ErrorResponse, HttpErrorCode};
use super::filters::{ContentTypeHeader, MethodAllowed};
use chrono::{Utc, Duration};
use std::ops::Add;
use uuid::Uuid;
use crate::user_service::UserService;
use sqlx::{MySqlPool, Pool, MySql, Connection, Acquire};
use futures::TryFutureExt;
use std::borrow::BorrowMut;


#[get("/profile")]
pub async fn profile(user: UserPrinciple, pool: web::Data<MySqlPool>) -> impl Responder {
    let pool_ref = pool.get_ref();
    //let result = &mut pool.acquire().await.unwrap();
    let mut user_service = UserService::new(pool_ref);
    let option = user_service.fetch_by_email(&user.email.unwrap()).await;
    option
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/user/")
        .service(profile));
}