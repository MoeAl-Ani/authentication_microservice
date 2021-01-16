use std::{fs, process};

use serde::{Deserialize, Serialize};
use actix_web::client::{Client, SendRequestError, ClientResponse, JsonPayloadError, Connector};
use actix_web::web::Bytes;
use actix_web::dev::Payload;
use actix_web::error::PayloadError;
use futures::future::TryFutureExt;
use std::sync::Arc;
use futures::{FutureExt, AsyncReadExt};
use reqwest;
use reqwest::{Url, Response, Error};
use openssl::ssl::{SslConnector, SslMethod};
use async_trait::async_trait;
use log::error;
use crate::services::jwt_service::{JwtClaims, SessionType, issue};
use chrono::{Utc, Duration};
use std::ops::Add;

#[async_trait]
pub trait BaseOAuth20Service {
    type ExternalAccount;
    fn get_authorization_url(&self) -> String;
    async fn get_access_token(&self, code: &String) -> String;
    async fn get_account_details(&self, access_token: &String) -> Option<Self::ExternalAccount>;
}

pub struct FacebookAuthenticationService {
    config: FacebookConfiguration
}

impl FacebookAuthenticationService {
    pub fn new() -> Self {
        FacebookAuthenticationService {
            config: FacebookConfiguration::new()
        }
    }
}

#[async_trait]
impl BaseOAuth20Service for FacebookAuthenticationService {
    type ExternalAccount = ExternalAccount;

    /// return this to the caller (client)
    fn get_authorization_url(&self) -> String {
        /// fbauth.getauthurl
        FacebookOAuth20Builder::new(&self.config.client_secret, &self.config.client_id)
            .scope("email".to_string())
            .redirect_url(self.config.callback_url.clone())
            .state(generate_state())
            .build_step1()
    }

    /// fetch auth token by code
    async fn get_access_token(&self, code: &String) -> String {
        let url = FacebookOAuth20Builder::new(&self.config.client_secret, &self.config.client_id)
            .scope("email".to_string())
            .redirect_url(self.config.callback_url.clone())
            .code(code.clone())
            .build_step2();
        let response = reqwest::get(url.parse::<Url>().unwrap()).await;
        match response {
            Ok(res) => {
                let data = res.text().await.unwrap();
                let result: FacebookAccessTokenResponse = serde_json::from_str(&data).unwrap();
                result.access_token
            }
            Err(err) => {
                error!("error = {}", err);
                "".to_string()
            }
        }
    }

    /// fetch account details using access_token
    async fn get_account_details(&self, access_token: &String) -> Option<Self::ExternalAccount> {
        /// &access_token
        let profile_url = &mut self.config.profile_url.clone();
        let access_suffix = format!("&access_token={}", access_token.as_str());
        profile_url.push_str(access_suffix.as_str());
        let response = reqwest::get(profile_url.parse::<Url>().unwrap()).await;
        match response {
            Ok(res) => {
                let data = res.text().await.unwrap();
                let mut result: ExternalAccount = serde_json::from_str(&data).unwrap();
                result.access_token = Some(access_token.clone());
                Some(result)
            }
            Err(err) => {
                error!("error = {}", err);
                None
            }
        }

    }
}

#[derive(Deserialize)]
struct FacebookAccessTokenResponse {
    access_token: String
}

#[derive(Deserialize)]
pub struct ExternalAccount {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub access_token: Option<String>,
}

impl ExternalAccount {
    pub fn new() -> Self {
        ExternalAccount {
            first_name: None,
            last_name: None,
            email: "".to_string(),
            access_token: Some("".to_string())
        }
    }
}

#[derive(Deserialize, Debug)]
struct FacebookConfiguration {
    client_id: String,
    client_secret: String,
    scope: String,
    callback_url: String,
    profile_url: String,
}

impl FacebookConfiguration {
    fn new() -> Self {
        let facebook_config = fs::read_to_string("./facebook_configuration.json").unwrap_or_else(|err| {
            eprintln!("error reading file {}", err);
            process::exit(1);
        });

        let fb_config: FacebookConfiguration = serde_json::from_value(facebook_config.parse().unwrap()).unwrap_or_else(|err| {
            eprintln!("error deserializing file content {}", err);
            process::exit(1);
        });
        fb_config
    }
}

struct FacebookOAuth20Builder<'a> {
    scope: Option<String>,
    redirect_url: Option<String>,
    state: Option<String>,
    client_secret: &'a String,
    client_id: &'a String,
    code: Option<String>
}

impl <'a> FacebookOAuth20Builder<'a> {
    fn new(client_secret: &'a String, client_id: &'a String) -> Self {
        FacebookOAuth20Builder {
            client_id,
            client_secret,
            scope: Some(format!("")),
            redirect_url: Some(format!("")),
            state: Some(format!("")),
            code: None
        }
    }
    fn scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
        self
    }

    fn redirect_url(mut self, url: String) -> Self {
        self.redirect_url = Some(url);
        self
    }

    fn state(mut self, state: String) -> Self {
        self.state = Some(state);
        self
    }

    fn code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }
    fn build_step1(&self) -> String {
        let mut base = "https://www.facebook.com/v9.0/dialog/oauth?".to_string();
        println!("{}", self.client_id);
        base.push_str(format!("client_id={}", self.client_id).as_str());

        match self.scope {
            None => {
                panic!(" no email supplied");
            }
            Some(ref scope) => {
                base.push_str(format!("&scope={}", scope).as_str());
            }
        };

        match self.state {
            None => {
                panic!(" no state supplied");
            }
            Some(ref state) => {
                base.push_str(format!("&state={}", state).as_str());
            }
        };

        match self.redirect_url {
            None => {
                panic!(" no state supplied");
            }
            Some(ref url) => {
                base.push_str(format!("&redirect_uri={}", url).as_str());
            }
        };
        base
    }

    fn build_step2(&self) -> String {
        let mut base = "https://graph.facebook.com/v9.0/oauth/access_token?".to_string();
        println!("{}", self.client_id);
        base.push_str(format!("client_id={}", self.client_id).as_str());
        base.push_str(format!("&client_secret={}", self.client_secret).as_str());

        match self.redirect_url {
            None => {
                panic!(" no state supplied");
            }
            Some(ref url) => {
                base.push_str(format!("&redirect_uri={}", url).as_str());
            }
        };

        match self.code {
            None => {
                panic!(" no code supplied");
            }
            Some(ref url) => {
                base.push_str(format!("&code={}", url).as_str());
            }
        };
        base
    }
}

fn generate_state() -> String {
    let mut claims = JwtClaims {
        aud: None,
        exp: Utc::now().add(Duration::minutes(1)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
        issuer: Some("infotamia.com".to_string()),
        jwt_id: Some(uuid::Uuid::new_v4().to_string()),
        sub: Some(uuid::Uuid::new_v4().to_string()),
        access_token: None,
        session_type: None,
    };

    issue(&mut claims)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_auth_service() {
        let service = FacebookAuthenticationService::new();
        let url = service.get_authorization_url();
        println!("url = {}", url)
    }

    #[test]
    fn test_load_fb_config() {
        let configuration = FacebookConfiguration::new();
        assert_eq!(configuration.scope, "email")
    }

    #[test]
    fn test_facebook_oauth_builder() {
        let client_secret = "secret".to_string();
        let client_id = "id".to_string();
        let authorization_url = FacebookOAuth20Builder::new(&client_secret, &client_id)
            .scope("email".to_string())
            .state("cunt".to_string())
            .redirect_url("someurl".to_string())
            .build_step1();
        assert_eq!(authorization_url, "https://www.facebook.com/v9.0/dialog/oauth?client_id=id&scope=email&state=cunt&redirect_uri=someurl")
    }
}

