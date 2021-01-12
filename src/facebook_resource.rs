use actix_web::{HttpResponse, Responder, get, web, Error};
use crate::oauth::{FacebookAuthenticationService, BaseOAuth20Service, ExternalAccount};
use crate::error_base::{HttpErrorCode, ErrorResponse};
use crate::jwt_service::{JwtClaims, SessionType};
use crate::jwt_service;
use chrono::{Utc, Duration};
use std::ops::Add;
use uuid::Uuid;
use serde::Deserialize;
use mysql::{Pool, Value};
use mysql::prelude::{TextQuery, Queryable};
use mysql::params::Params;
use mysql::params;
use mysql::prelude::*;
use std::collections::HashMap;
use crate::entities::UserEntity;


#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
    state: String
}
/// step one login
/// return a url String.
#[get("/login1")]
pub async fn login_step_1(auth_service: web::Data<FacebookAuthenticationService>) -> impl Responder {
    auth_service.get_authorization_url()
}

/// general echo resource
#[get("/callback")]
pub async fn login_step_2(
    mut auth_service: web::Data<FacebookAuthenticationService>,
    query: web::Query<CallbackQuery>,
    pool: web::Data<Pool>) -> Result<HttpResponse, HttpErrorCode> {
    let state_option = jwt_service::verify(&query.state);
    match state_option {
        None => {
            return Err(HttpErrorCode::UnAuthorized {message : ErrorResponse {message: "no user found".to_string(), error_code : "unauthorized".to_string()}})
        }
        Some(_) => {

        }
    };
    let access_token = auth_service.get_access_token(&query.code).await;
    let user_profile_optional = auth_service.get_account_details(&access_token).await;
    match user_profile_optional {
        None => {
            Err(HttpErrorCode::UnAuthorized {message : ErrorResponse {message: "no user found".to_string(), error_code : "unauthorized".to_string()}})
        }
        Some(user) => {
            let mut conn = pool.get_conn().unwrap().unwrap();
            let vec_entities = vec![UserEntity::from_external_account(&user)];
            let statement = conn.prep(r"INSERT INTO user(first_name, last_name, email, phone_number, language_id) VALUES(:first_name,:last_name,:email,:phone_number,:language_id)").unwrap();
            let result = conn.exec_batch(&statement,
                                         vec_entities.iter().map(|e| mysql::params! {
                   "first_name" => &e.first_name,
                   "last_name" => &e.last_name,
                   "email" => &e.email,
                   "phone_number" => &e.phone_number,
                   "language_id" => e.language_id
                })).unwrap_or_else(|err| {});
            conn.close(statement);
            /// TODO insert user info to db if user not exist
            /// return a jwt for the caller
            /// create claims
            /// issue
            let mut claims = JwtClaims {
                aud: None,
                exp: Utc::now().add(Duration::days(1)).timestamp() as usize,
                iat: Utc::now().timestamp() as usize,
                issuer: Some("infotamia.com".to_string()),
                jwt_id: Some(Uuid::new_v4().to_string()),
                sub: Some(user.email.clone()),
                access_token: Some(user.access_token.unwrap().clone()),
                session_type: Some(SessionType::USER),
            };
            let jwt = jwt_service::issue(&mut claims);
            let response = HttpResponse::Ok().header("Authorization", format!("bearer {}", jwt)).finish();
            Ok(response)
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/iot/auth2/facebook/")
        .service(login_step_1)
        .service(login_step_2));
}