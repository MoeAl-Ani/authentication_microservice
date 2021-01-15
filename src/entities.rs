use crate::oauth::ExternalAccount;
use serde::{Serialize, Deserialize};
use actix_web::{Responder, HttpRequest, Error, HttpResponse};
use actix_web::body::Body;
use futures::future::{ready, Ready, ok};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserEntity {
    pub id: Option<u32>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub phone_number: String,
    pub language_id: i32
}

impl UserEntity {
    pub fn from_external_account(external_account: &ExternalAccount) -> Self {
        UserEntity {
            first_name: external_account.first_name.clone(),
            last_name: external_account.last_name.clone(),
            email: external_account.email.clone(),
            phone_number: "0403231145".to_string(),
            id: None,
            language_id: 1
        }
    }
}

impl Responder for UserEntity {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Self::Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}