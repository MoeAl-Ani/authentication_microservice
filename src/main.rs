use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

use actix_web::{App, get, guard, HttpResponse, HttpServer, post, Responder, web, Error, middleware};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::middleware::Logger;
use env_logger::Env;
use log::{debug, error, info, Level, log_enabled};

use crate::filters::{ContentTypeHeader, MethodAllowed};
use crate::jwt_service::SessionType;
use std::fmt::Display;
use serde::export::Formatter;
use rand::prelude::*;
use futures::future::{ok, Ready};
use tokio::macros::support::Future;


mod echo_resource;
mod error_base;
mod filters;
mod jwt_service;

#[derive(Debug, Clone)]
pub struct UserPrinciple {
    email: Option<String>,
    session_type: Option<SessionType>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(echo_resource::AppStateWithCounter::new());
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(Logger::default())
            .wrap(filters::AuthFilter)
            .data_factory(|| -> Ready<Result<String, Error>>{
                let x: u8 = random();
                ok(format!("Thread-{}", x))
            })
            .app_data(counter.clone())
            .configure(echo_resource::config)
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
