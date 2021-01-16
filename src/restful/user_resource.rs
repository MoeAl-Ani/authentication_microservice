use std::borrow::BorrowMut;
use std::ops::Add;
use std::sync::{Mutex, RwLock};

use actix_web::{App, Error, get, guard, HttpRequest, HttpResponse, HttpServer, post, Responder, web};
use actix_web::body::Body;
use actix_web::web::Data;
use chrono::{Duration, Utc};
use futures::future::{ready, Ready};
use futures::TryFutureExt;
use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, Connection, MySql, MySqlPool, Pool};
use uuid::Uuid;

use crate::{main, UserPrinciple};
use crate::daos::user_dao;
use crate::services::jwt_service::SessionType;
use crate::services::user_service::UserService;

use crate::exceptions::error_base::{ErrorResponse, HttpErrorCode};
use crate::filters::authentication_filter::{ContentTypeHeader, MethodAllowed};

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