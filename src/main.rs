use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

use actix_web::{App, Error, get, guard, HttpResponse, HttpServer, middleware, post, Responder, web};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::middleware::Logger;
use env_logger::Env;
use futures::future::{ok, Ready};
use log::{debug, error, info, Level, log_enabled};
use rand::prelude::*;
use serde::export::Formatter;

use db::connection_pool_manager::PoolInstantiate;
use filters::{authentication_filter, cors_filter};
use filters::authentication_filter::{ContentTypeHeader, MethodAllowed};
use ouath::oauth::FacebookAuthenticationService;
use restful::{echo_resource, facebook_resource, user_resource};
use services::jwt_service::SessionType;
use std::iter::Map;
use rust_srp::SrpServer;
use crate::restful::srp_resource;

mod daos;
mod entities;
mod restful;
mod filters;
mod db;
mod services;
mod exceptions;
mod ouath;

#[derive(Debug, Clone)]
pub struct UserPrinciple {
    email: Option<String>,
    session_type: Option<SessionType>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let srp_session_management:HashMap<String, SrpServer> = HashMap::new();
    let srp_session_management = web::Data::new(Mutex::new(srp_session_management));
    let counter = web::Data::new(echo_resource::AppStateWithCounter::new());
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let pool = PoolInstantiate::init().await;
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(authentication_filter::AuthFilter)
            .wrap(cors_filter::CorsFilter)
            .data_factory(|| -> Ready<Result<String, Error>>{
                let x: u8 = random();
                ok(format!("Thread-{}", x))
            })
            .app_data(srp_session_management.clone())
            .app_data(counter.clone())
            .data(pool.clone())
            .data(FacebookAuthenticationService::new())
            .configure(echo_resource::config)
            .configure(facebook_resource::config)
            .configure(user_resource::config)
            .configure(srp_resource::config)
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
