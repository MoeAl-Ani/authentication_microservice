use std::sync::{Mutex, RwLock};

use actix_web::{App, Error, get, guard, HttpRequest, HttpResponse, HttpServer, post, Responder, web};
use actix_web::body::Body;
use actix_web::web::Data;
use futures::future::{ready, Ready};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{main, UserPrinciple};
use crate::jwt_service::SessionType;

use super::error_base::{ErrorResponse, HttpErrorCode};
use super::filters::{ContentTypeHeader, MethodAllowed};
use mysql::{Pool, Value, Row, Conn};
use mysql::prelude::{TextQuery, Queryable};
use mysql::params::Params;
use mysql::params;
use mysql::prelude::*;

#[get("/profile")]
pub async fn profile(user: UserPrinciple, pool: web::Data<Pool>) -> impl Responder {
    let mut conn = pool.get_conn().unwrap();
    let statement = conn.prep("SELECT * from user where user.email like :email").unwrap();
    let option: Option<Row> = conn.exec_first(&statement, mysql::params! {
        "email" => &user.email
    }).unwrap();
    conn.close(statement);
    "fetched"
}


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/user/")
        .service(profile));
}