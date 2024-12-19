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

pub trait ApiError: std::fmt::Debug + Send + Sync {
    fn status(&self) -> rocket::http::Status;
    fn message(&self) -> String;
}

impl<'r> Responder<'r, 'static> for Box<dyn ApiError> {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        rocket::response::Response::build()
            .status(self.status())
            .sized_body(self.message().len(), std::io::Cursor::new(self.message()))
            .ok()
    }
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

#[derive(Debug)]
pub enum ImageError {
    DecodeBytesError,              // 图片解码失败
    EncodeBytesError,              // 图片编码失败
    FailedStartAddWatermark,       // 启动水印脚本失败
    WatermarkProcessError(String), // 水印处理失败，包含 Python 错误信息
    JsonParseError,                // 解析 Python 脚本返回的 JSON 数据失败
    IOError(String),               // 通用 I/O 错误，包含详细描述
    DatabaseError(rbatis::Error),
    IpfsError(String),
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ImageError::DecodeBytesError => write!(f, "Failed to decode the image bytes"),
            ImageError::EncodeBytesError => write!(f, "Failed to encode the image bytes"),
            ImageError::FailedStartAddWatermark => {
                write!(f, "Failed to start the watermark process")
            }
            ImageError::WatermarkProcessError(err) => {
                write!(f, "Watermark processing error: {}", err)
            }
            ImageError::JsonParseError => write!(f, "Failed to parse JSON data"),
            ImageError::IOError(err) => write!(f, "I/O error: {}", err),
            ImageError::DatabaseError(err) => write!(f, "Database error: {}", err),
            ImageError::IpfsError(err) => write!(f, "IPFS error: {}", err),
        }
    }
}

impl ApiError for ImageError {
    fn status(&self) -> rocket::http::Status {
        match self {
            ImageError::DecodeBytesError => rocket::http::Status::BadRequest,
            ImageError::EncodeBytesError => rocket::http::Status::BadRequest,
            ImageError::FailedStartAddWatermark => rocket::http::Status::InternalServerError,
            ImageError::WatermarkProcessError(_) => rocket::http::Status::InternalServerError,
            ImageError::JsonParseError => rocket::http::Status::BadRequest,
            ImageError::IOError(_) => rocket::http::Status::InternalServerError,
            ImageError::DatabaseError(_) => rocket::http::Status::InternalServerError,
            ImageError::IpfsError(_) => rocket::http::Status::InternalServerError,
        }
    }

    fn message(&self) -> String {
        self.to_string() // 使用 fmt::Display 的实现作为错误消息
    }
}

#[derive(Debug)]
pub enum BlockchainError {
    SendTransactionError(String),  // 发送交易失败
    WatchTransactionError(String), // 监听交易哈希失败
    ContractCallError(String),     // 合约调用失败
    ContractInitializeError(String),
}

impl fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockchainError::SendTransactionError(err) => {
                write!(f, "Failed to send transaction: {}", err)
            }
            BlockchainError::WatchTransactionError(err) => {
                write!(f, "Failed to watch transaction: {}", err)
            }
            BlockchainError::ContractCallError(err) => {
                write!(f, "Contract call error: {}", err)
            }
            BlockchainError::ContractInitializeError(err) => {
                write!(f, "Contract initialize error: {}", err)
            }
        }
    }
}

impl ApiError for BlockchainError {
    fn status(&self) -> Status {
        match self {
            BlockchainError::SendTransactionError(_) => Status::BadRequest,
            BlockchainError::WatchTransactionError(_) => Status::InternalServerError,
            BlockchainError::ContractCallError(_) => Status::InternalServerError,
            BlockchainError::ContractInitializeError(_) => Status::BadRequest,
        }
    }

    fn message(&self) -> String {
        self.to_string() // 使用 fmt::Display 的实现作为错误消息
    }
}
