use std::fmt::Error;
use std::fmt::Display;
use std::fmt::Debug;
use actix_web::{
    dev::HttpResponseBuilder, error, get, http::header, http::StatusCode, App, HttpResponse,
};
use serde::export::Formatter;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
    pub error_code: String
}

#[derive(Debug)]
pub enum ErrorCode {
    MissingUserId
}
#[derive(Debug)]
pub enum HttpErrorCode {
    BadRequest {message: ErrorResponse}
}

impl Display for HttpErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpErrorCode::BadRequest { message: error_detail } => {
                write!(f, "({}, {})", error_detail.message, error_detail.error_code)
            }
        }
    }
}


impl error::ResponseError for HttpErrorCode {
    fn status_code(&self) -> StatusCode {
        match *self {
            HttpErrorCode::BadRequest { .. } => {
                StatusCode::BAD_REQUEST
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
}