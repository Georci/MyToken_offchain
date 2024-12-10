use rbatis::Error;
use rocket::http::Status;
use rocket::response::Responder;
use rocket::serde::json::Json;
use std::fmt;

pub enum TxError {
    ExecuteError,
}

#[derive(Debug)]
pub enum AppError {
    IoError(std::io::Error),
    DatabaseError(rbatis::Error),
    Custom(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Custom(err)
    }
}

// 实现 Responder
impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let (status, body) = match self {
            AppError::IoError(_) => (
                Status::InternalServerError,
                "Internal server error".to_string(),
            ),
            AppError::DatabaseError(err) => (Status::InternalServerError, err.to_string()),
            AppError::Custom(msg) => (Status::InternalServerError, msg),
        };

        rocket::response::Response::build()
            .status(status)
            .sized_body(body.len(), std::io::Cursor::new(body))
            .ok()
    }
}
