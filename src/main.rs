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
use tokio::macros::support::Future;

use crate::filters::{ContentTypeHeader, MethodAllowed};
use crate::jwt_service::SessionType;
use crate::oauth::FacebookAuthenticationService;
use crate::connection_pool_manager::PoolInstantiate;

mod echo_resource;
mod error_base;
mod filters;
mod jwt_service;
mod cors_filter;
mod user_dao;
mod oauth;
mod facebook_resource;
mod connection_pool_manager;
mod user_resource;

#[derive(Debug, Clone)]
pub struct UserPrinciple {
    email: Option<String>,
    session_type: Option<SessionType>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(echo_resource::AppStateWithCounter::new());
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let pool = web::Data::new(PoolInstantiate::init());
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(filters::AuthFilter)
            .wrap(cors_filter::CorsFilter)
            .data_factory(|| -> Ready<Result<String, Error>>{
                let x: u8 = random();
                ok(format!("Thread-{}", x))
            })
            .app_data(counter.clone())
            .app_data(pool.clone())
            .data(FacebookAuthenticationService::new())
            .configure(echo_resource::config)
            .configure(facebook_resource::config)
            .configure(user_resource::config)
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
