use actix_web::{dev::RequestHead, guard::Guard, http, HttpResponse};

pub struct ContentTypeHeader;
pub struct MethodAllowed;
use log::debug;
/// Guard filter for content type
impl Guard for ContentTypeHeader {
    fn check(&self, req: &RequestHead) -> bool {
        req.headers().contains_key(http::header::CONTENT_TYPE)
    }
}

/// Guard filter for allowed methods
impl Guard for MethodAllowed {
    fn check(&self, req: &RequestHead) -> bool {
        let inner = req.method.as_str();
        match inner {
            //"GET" => {true}
            "POST" => {true}
            "PATCH" => {true}
            "DELETE" => {true}
            "OPTION" => {true}
            _ => {false}
        }
    }
}