///错误处理
/// 1.定义程序中的可恢复错误与不可恢复错误
/// 2.定义自己的错误类型
/// 3.判断哪些错误需要在当前函数中处理，哪些错误需要向上传递
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

pub trait ApiError: std::fmt::Debug + Send + Sync {
    fn status(&self) -> rocket::http::Status;
    fn message(&self) -> String;
}

#[derive(Debug)]
pub enum UserError {
    UserAlreadyExists,
    UserNotFound,
    InvalidPassword,
    DatabaseError(rbatis::Error),
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserError::UserAlreadyExists => {
                write!(f, "User already exists")
            }
            UserError::UserNotFound => {
                write!(f, "User not found")
            }
            UserError::InvalidPassword => {
                write!(f, "Invalid password")
            }
            UserError::DatabaseError(e) => {
                write!(f, "Database error: {}", e)
            }
        }
    }
}

impl ApiError for UserError {
    fn status(&self) -> Status {
        match self {
            UserError::UserAlreadyExists => Status::InternalServerError,
            UserError::UserNotFound => Status::InternalServerError,
            UserError::InvalidPassword => Status::BadRequest,
            UserError::DatabaseError(_) => Status::InternalServerError,
        }
    }

    fn message(&self) -> String {
        match self {
            UserError::UserAlreadyExists => "User already exists".to_string(),
            UserError::UserNotFound => "User not found".to_string(),
            UserError::InvalidPassword => "Invalid password".to_string(),
            UserError::DatabaseError(e) => format!("Database error: {}", e),
        }
    }
}

impl<'r> Responder<'r, 'static> for UserError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let status = self.status();
        let body = self.message();

        rocket::response::Response::build()
            .status(status)
            .sized_body(body.len(), std::io::Cursor::new(body))
            .ok()
    }
}

#[derive(Debug)]
pub enum RequestError {
    EmptyUsername,
    TooShortPassword,
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestError::EmptyUsername => {
                write!(f, "Empty username")
            }
            RequestError::TooShortPassword => {
                write!(f, "password too short, at least 3 characters")
            }
        }
    }
}

impl ApiError for RequestError {
    fn status(&self) -> rocket::http::Status {
        match self {
            RequestError::EmptyUsername => rocket::http::Status::BadRequest,
            RequestError::TooShortPassword => rocket::http::Status::BadRequest,
        }
    }
    fn message(&self) -> String {
        match self {
            RequestError::EmptyUsername => "Empty username".to_string(),
            RequestError::TooShortPassword => "Password too short".to_string(),
        }
    }
}

impl<'r> Responder<'r, 'static> for Box<dyn ApiError> {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        rocket::response::Response::build()
            .status(self.status())
            .sized_body(self.message().len(), std::io::Cursor::new(self.message()))
            .ok()
    }
}
