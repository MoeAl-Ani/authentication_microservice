use actix_web::{HttpResponse, Responder, post, web, Error};
use crate::ouath::oauth::{FacebookAuthenticationService, BaseOAuth20Service, ExternalAccount};
use crate::exceptions::error_base::{HttpErrorCode, ErrorResponse};
use crate::services::jwt_service::{JwtClaims, SessionType};
use crate::services::jwt_service;
use chrono::{Utc, Duration};
use std::ops::Add;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use crate::services::user_service::UserService;
use sqlx::{MySql, Pool, MySqlPool};
use crate::entities::user_entity::UserEntity;
use crate::entities::srp::srp_entities::{SrpStep1Request, SrpStep2Request, SrpStep2Response, SrpStep1Response};
use std::borrow::Borrow;
use rust_srp::{SrpServer, SrpConfig};
use std::sync::{Mutex, PoisonError, MutexGuard};
use std::collections::hash_map::RandomState;
use rust_srp::bigint_helper::{convert_to_bigint, generate_random_256bit_bigint};
use num_bigint::BigUint;


#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
    state: String
}
/// step one login
/// receive public key A and identity
/// returns salt, B
#[post("/1")]
pub async fn login_step_1(
    srp_req: web::Json<SrpStep1Request>,
    srp_session_map: web::Data<Mutex<HashMap<String, SrpServer>>>,
    pool: web::Data<Pool<MySql>>) -> Result<HttpResponse, HttpErrorCode> {
    let req = srp_req.0.borrow();
    let identity = req.identity.clone();
    let public_a_str = req.public_a_str.clone();
    let n = BigUint::parse_bytes(b"B97F8C656C3DF7179C2B805BBCB3A0DC4B0B6926BF66D0A3C63CF6015625CAF9A4DB4BBE7EB34253FAB0E475A6ACFAE49FD5F22C47A71B5532911B69FE7DF4F8ACEE2F7785D75866CF6D213286FC7EBBBE3BE411ECFA10A70F0C8463DC1182C6F9B6F7666C8691B3D1AB6FD78E9CBF8AAE719EA75CA02BE87AE445C698BF0413", 16).unwrap();
    let g = BigUint::parse_bytes(b"2", 10).unwrap();
    let public_a = rust_srp::bigint_helper::convert_to_bigint(public_a_str.as_bytes(), 10);

    let pool_ref = pool.get_ref();
    //let result = &mut pool.acquire().await.unwrap();
    let mut user_service = UserService::new(pool_ref);
    let option = user_service.fetch_by_email(&identity).await;
    let mut srp_server = SrpServer::new(public_a.unwrap(), n, g);
    match option {
        None => {
            Err(HttpErrorCode::UnAuthorized {message : ErrorResponse {message: "unknown".to_string(), error_code : "unauthorized".to_string()}})
        }
        Some(user) => {
            let salt_str = user.salt.clone().unwrap();
            let salt = rust_srp::bigint_helper::convert_to_bigint(salt_str.as_bytes(), 10).unwrap();
            let verifier_str = user.verifier.clone().unwrap();
            let verifier = rust_srp::bigint_helper::convert_to_bigint(verifier_str.as_bytes(), 10).unwrap();
            let srp_1_result = srp_server.step_1(identity.clone(), salt, verifier);
            match srp_1_result {
                Ok(public_b) => {
                    match srp_session_map.lock() {
                        Ok(mut sessions) => {
                            sessions.insert(user.email.clone(), srp_server);
                            let srp_step1_response = SrpStep1Response {
                                salt_str,
                                public_b_str: public_b.to_string()
                            };

                            let body = serde_json::to_string(&srp_step1_response).unwrap();
                            Ok(HttpResponse::Ok()
                                .content_type("application/json")
                                .body(body))
                        }
                        Err(err) => {
                            Err(HttpErrorCode::UnAuthorized {message : ErrorResponse {message: err.to_string(), error_code : "unauthorized".to_string()}})
                        }
                    }
                }
                Err(err) => {
                    match srp_session_map.lock() {
                        Ok(mut sessions) => {
                            sessions.remove(identity.as_str());
                        }
                        Err(_) => {}
                    }
                    Err(HttpErrorCode::UnAuthorized {message : ErrorResponse {message: err.to_string(), error_code : "unauthorized".to_string()}})
                }
            }
        }
    }

}

/// srp step 2 validate client m1 evidence and generate server m2 evidence
#[post("/2")]
pub async fn login_step_2(
    srp_req: web::Json<SrpStep2Request>,
    pool: web::Data<Pool<MySql>>,
    srp_session_map: web::Data<Mutex<HashMap<String, SrpServer>>>) -> Result<HttpResponse, HttpErrorCode> {
    let identity = srp_req.identity.clone();
    let m1_str = srp_req.m1_str.clone();
    match srp_session_map.lock() {
        Ok(mut sessions) => {
            let m1 = convert_to_bigint(m1_str.as_bytes(), 10).unwrap();
            let session = sessions.remove(identity.as_str()).unwrap();
            match session.step_2(m1.clone()) {
                Ok(m2) => {
                    let srp2response = SrpStep2Response {
                        m2_str: m2.to_string()
                    };

                    let body = serde_json::to_string(&srp2response).unwrap();
                    Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body(body))
                }
                Err(err) => {
                    Err(HttpErrorCode::UnAuthorized {message : ErrorResponse {message: err.to_string(), error_code : "unauthorized".to_string()}})
                }
            }
        }
        Err(err) => {
            Err(HttpErrorCode::UnAuthorized {message : ErrorResponse {message: err.to_string(), error_code : "unauthorized".to_string()}})
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/srp/")
        .service(login_step_1)
        .service(login_step_2));
}

#[derive(Deserialize, Serialize)]
struct Ass {
    name: String
}

#[cfg(test)]
mod test {
    use std::ops::Add;
    use std::sync::Mutex;
    use std::thread;
    use std::time::SystemTime;

    use actix_web::{App, test, web};
    use actix_web::http::{StatusCode, Method};
    use chrono::{Duration, NaiveDateTime, Timelike, Utc};
    use env_logger::Env;
    use futures::task::SpawnExt;

    use crate::services::jwt_service::{issue, SessionType};
    use crate::restful::{echo_resource, srp_resource};
    use crate::filters::{authentication_filter, cors_filter};
    use crate::db::connection_pool_manager::PoolInstantiate;
    use rust_srp::{SrpClient, SrpServer};
    use std::collections::HashMap;
    use num_bigint::BigUint;
    use crate::entities::srp::srp_entities::{SrpStep1Request, SrpStep1Response, SrpStep2Response, SrpStep2Request};
    use serde::{Deserialize, Serialize};
    use serde_json;
    use rust_srp::bigint_helper::convert_to_bigint;

    #[actix_rt::test]
    async fn test_srp_server_flow() {
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var("RUST_BACKTRACE", "1");
        env_logger::try_init();
        let srp_session_management:HashMap<String, SrpServer> = HashMap::new();
        let srp_session_management = web::Data::new(Mutex::new(srp_session_management));
        let pool = PoolInstantiate::init().await;
        let mut app = test::init_service(App::new()
            .wrap(authentication_filter::AuthFilter)
            .wrap(cors_filter::CorsFilter)
            .app_data(srp_session_management.clone())
            .data(pool.clone())
            .configure(srp_resource::config)).await;

        // client
        let n = BigUint::parse_bytes(b"B97F8C656C3DF7179C2B805BBCB3A0DC4B0B6926BF66D0A3C63CF6015625CAF9A4DB4BBE7EB34253FAB0E475A6ACFAE49FD5F22C47A71B5532911B69FE7DF4F8ACEE2F7785D75866CF6D213286FC7EBBBE3BE411ECFA10A70F0C8463DC1182C6F9B6F7666C8691B3D1AB6FD78E9CBF8AAE719EA75CA02BE87AE445C698BF0413", 16).unwrap();
        let g = BigUint::parse_bytes(b"2", 10).unwrap();
        let mut client = SrpClient::new(n, g);
        let public_a = client.step_1("mohammedalanny@gmail.com".to_string(), "12345678".to_string());
        let srp1_request = SrpStep1Request {
            identity: format!("{}", "mohammedalanny@gmail.com"),
            public_a_str: public_a.unwrap().to_string()
        };

        let body = serde_json::to_string(&srp1_request).unwrap();

        let req = test::TestRequest::with_header("content-type", "application/json")
            .uri("/srp/1")
            .set_json(&srp1_request)
            .method(Method::POST)
            .to_request();
        let mut resp = test::read_response(&mut app, req).await;
        let srp1_response: SrpStep1Response = serde_json::from_slice(resp.as_ref()).unwrap();
        let salt = convert_to_bigint(srp1_response.salt_str.clone().as_bytes(), 10).unwrap();
        let public_b = convert_to_bigint(srp1_response.public_b_str.clone().as_bytes(), 10).unwrap();

        let m1 = client.step_2(salt, public_b).unwrap();
        let srp2_request = SrpStep2Request {
            identity: format!("{}", "mohammedalanny@gmail.com"),
            m1_str: m1.to_string()
        };

        let req = test::TestRequest::with_header("content-type", "application/json")
            .uri("/srp/2")
            .set_json(&srp2_request)
            .method(Method::POST)
            .to_request();
        let mut resp = test::read_response(&mut app, req).await;
        let srp2_response: SrpStep2Response = serde_json::from_slice(resp.as_ref()).unwrap();
        client.step_3(convert_to_bigint(srp2_response.m2_str.as_bytes(), 10).unwrap()).unwrap();


    }
}