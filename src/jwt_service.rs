use chrono::{DateTime, NaiveDateTime, Utc};
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, TokenData, Validation};
use jsonwebtoken::errors::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum SessionType {
    USER, GUEST, SYSADMIN
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum AuthenticationProvider {
    FACEBOOK, GOOGLE, TWITTER, MANUAL, APPLE, GUEST
}


#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub jwt_id: Option<String>,
    pub sub: Option<String>,
    pub aud: Option<String>,
    pub issuer: Option<String>,
    pub session_type: Option<SessionType>,
    pub access_token: Option<String>,
    pub iat: usize,
    pub exp: usize

}

pub fn issue(claims: &mut JwtClaims) -> String {
    let header = Header::new(Algorithm::HS256);
    encode(&header, claims, &EncodingKey::from_secret("secret".as_ref())).unwrap()
}

pub fn verify(token: &String) -> Option<JwtClaims> {
    let result = decode::<JwtClaims>(token, &DecodingKey::from_secret("secret".as_ref()), &Validation::default());
    match result {
        Ok(data) => {Some(data.claims)}
        Err(_) => {None}
    }
}

#[cfg(test)]
mod test {
    use std::ops::Add;
    use std::time::SystemTime;

    use chrono::{Duration, NaiveDateTime, Timelike, Utc};

    use super::*;

    #[test]
    fn test_time() {
        let rfc3339 = NaiveDateTime::parse_from_str("2021-01-03 15:35:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let now: DateTime<Utc> = Utc::now();
        let i = now.timestamp_nanos();
        println!("x = {}", rfc3339.timestamp_nanos());
        println!("y = {}", now.timestamp_nanos());
    }


    #[test]
    fn test_issuing_verifying_jwt() {
        let mut claims = JwtClaims {
            aud: Some("".to_string()),
            exp: Utc::now().add(Duration::days(1)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            issuer: Some("infotamia".to_string()),
            jwt_id: Some("myid".to_string()),
            sub: Some("moe@gmail.com".to_string()),
            access_token: Some("sometoken".to_string()),
            session_type: Some(SessionType::USER)
        };

        let token = issue(&mut claims);
        print!("jwt = {}", token.clone());
        assert!(!token.is_empty());

        // verify it is correct
        let verified_claims = verify(&token).unwrap();
        assert_eq!(claims.sub.unwrap(), verified_claims.sub.unwrap());
        assert_eq!(claims.jwt_id.unwrap(), verified_claims.jwt_id.unwrap());
        assert_eq!(claims.issuer.unwrap(), verified_claims.issuer.unwrap());
        assert_eq!(verified_claims.session_type.unwrap(), SessionType::USER);
    }
}