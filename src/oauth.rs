use std::{fs, process};

use serde::{Deserialize, Serialize};

pub trait BaseOAuth20Service {
    type ExternalAccount;
    fn get_authorization_url(&self) -> &str;
    fn get_access_token(&self, code: &String) -> String;
    fn get_account_details(&self, access_token: &String) -> Option<Self::ExternalAccount>;
}

struct FacebookAuthenticationService;

impl FacebookAuthenticationService {
    fn new() -> Self {
        FacebookAuthenticationService {}
    }
}

impl BaseOAuth20Service for FacebookAuthenticationService {
    type ExternalAccount = ExternalAccount;

    /// return this to the caller (client)
    fn get_authorization_url(&self) -> &str {
        unimplemented!()
    }

    /// fetch auth token by code
    fn get_access_token(&self, code: &String) -> String {
        unimplemented!()
    }

    /// fetch account details using access_token
    fn get_account_details(&self, access_token: &String) -> Option<Self::ExternalAccount> {
        unimplemented!()
    }
}

struct ExternalAccount {
    first_name: Option<String>,
    last_name: Option<String>,
    email: String,
    access_token: String,
}

#[derive(Deserialize)]
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

struct FacebookOAuth20Builder {
    scope: Option<String>,
    redirect_url: Option<String>,
    state: Option<String>,
    client_secret: String,
    client_id: String,
}

impl FacebookOAuth20Builder {
    fn new(client_secret: String, client_id: String) -> Self {
        FacebookOAuth20Builder {
            client_id,
            client_secret,
            scope: Some(format!("")),
            redirect_url: Some(format!("")),
            state: Some(format!("")),
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
    fn build(&self) -> String {
        let mut base = "https://www.facebook.com/v9.0/dialog/oauth?".to_string();
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
                base.push_str(format!("&redirect_url={}", url).as_str());
            }
        };
        base
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_auth_service() {
        let service = FacebookAuthenticationService::new();
        let string = format!("ss");
        service.get_access_token(&string);
    }

    #[test]
    fn test_load_fb_config() {
        let configuration = FacebookConfiguration::new();
        assert_eq!(configuration.scope, "email")
    }

    #[test]
    fn test_facebook_oauth_builder() {
        let authorization_url = FacebookOAuth20Builder::new("secret".to_string(), "id".to_string())
            .scope("email".to_string())
            .state("cunt".to_string())
            .redirect_url("someurl".to_string())
            .build();
        assert_eq!(authorization_url, "https://www.facebook.com/v9.0/dialog/oauth?client_id=id&scope=email&state=cunt&redirect_url=someurl")
    }
}
