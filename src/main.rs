use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, guard};
use std::sync::Mutex;
use log::{debug, error, log_enabled, info, Level};


use actix_web::middleware::Logger;
use env_logger::Env;
use crate::filters::{ContentTypeHeader, MethodAllowed};

mod echo_resource;
mod error_base;
mod filters;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(echo_resource::AppStateWithCounter::new());
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(counter.clone())
            .configure(echo_resource::config)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
