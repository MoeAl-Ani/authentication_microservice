use actix_web::{HttpResponse, Responder, get, web, Error};
use crate::oauth::{FacebookAuthenticationService, BaseOAuth20Service, ExternalAccount};
use crate::error_base::{HttpErrorCode, ErrorResponse};
use crate::jwt_service::{JwtClaims, SessionType};
use crate::jwt_service;
use chrono::{Utc, Duration};
use std::ops::Add;
use uuid::Uuid;
use serde::Deserialize;

#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
    state: String
}
/// step one login
#[get("/login1")]
pub async fn login_step_1(auth_service: web::Data<FacebookAuthenticationService>) -> impl Responder {
    auth_service.get_authorization_url()
}

/// general echo resource
#[get("/callback")]
pub async fn login_step_2(
    mut auth_service: web::Data<FacebookAuthenticationService>, query: web::Query<CallbackQuery>) -> Result<HttpResponse, HttpErrorCode> {
    /// TODO validate state
    let state_option = jwt_service::verify(&query.state);
    match state_option {
        None => {
            return Err(HttpErrorCode::UnAuthorized {message : ErrorResponse {message: "no user found".to_string(), error_code : "unauthorized".to_string()}})
        }
        Some(data) => {

        }
    };
    let access_token = auth_service.get_access_token(&query.code).await;
    let user_profile_optional = auth_service.get_account_details(&access_token).await;
    //let user_profile_optional = Some(ExternalAccount::new());
    match user_profile_optional {
        None => {
            Err(HttpErrorCode::UnAuthorized {message : ErrorResponse {message: "no user found".to_string(), error_code : "unauthorized".to_string()}})
        }
        Some(user) => {
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