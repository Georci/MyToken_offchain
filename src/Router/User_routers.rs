use crate::DataBase::get_db;
use crate::DataBase::Users;
use crate::Error::{ApiError, RequestError, UserError};
use crate::UserInfo::Generate_address::generate_random_account;
use crate::UserInfo::Login::{login_user, register_user};
use rbatis::executor::Executor;
use rbatis::Error;
use rbatis::RBatis;
use rocket::form::FromForm;
use rocket::http::Status;
use rocket::response::{status, Responder};
use rocket::{launch, post, routes, tokio};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use validator::{Validate, ValidationErrors};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 1, message = "Username cannot be empty"))]
    username: String,
    #[validate(length(min = 3, message = "Password must be at least 3 characters long"))]
    password: String,
    company_name: String,
}

#[derive(Serialize, Deserialize)]
struct RegisterResponse {
    address: String,
    privatekey: String,
}

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

// 定义登录响应结构
#[derive(Serialize, Deserialize)]
struct LoginResponse {
    address: Option<String>,
    token: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error_code: String, // 错误码，例如 "UserAlreadyExists"
    message: String,    // 错误的详细描述，例如 "The username is already taken"
}

// 已测试接口
#[post("/register", format = "json", data = "<register_request>")]
pub async fn register(
    register_request: Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, Box<dyn ApiError>> {
    // Validate the incoming request data
    if let Err(validation_errors) = register_request.validate() {
        // Map `validation_errors` to `RequestError`
        if let Some(error) = map_validation_errors(&validation_errors) {
            return Err(Box::new(error));
        }
    }

    let result = register_user(
        &register_request.username,
        &register_request.password,
        &register_request.company_name,
    )
    .await;
    match result {
        Ok((address, private_key)) => Ok(Json(RegisterResponse {
            address,
            privatekey: private_key,
        })),
        // 返回错误信息和状态码
        Err(error) => Err(Box::new(error)),
    }
}

// 已测试接口
// Web API 处理函数
#[post("/login", format = "json", data = "<login_request>")]
pub async fn login(
    login_request: Json<LoginRequest>,
) -> Result<Json<LoginResponse>, Box<dyn ApiError>> {
    let result = login_user(&login_request.username, &login_request.password).await;
    match result {
        Ok((address, token)) => Ok(Json(LoginResponse {
            address: Some(address),
            token: Some(token),
        })),
        Err(error) => {
            eprintln!("Error: {}", error);
            Err(Box::new(error))
        }
    }
}

// Helper function to map `ValidationErrors` to `RequestError`
fn map_validation_errors(errors: &ValidationErrors) -> Option<RequestError> {
    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            match field {
                "username" => return Some(RequestError::EmptyUsername),
                "password" => {
                    if error.code == "length" {
                        return Some(RequestError::TooShortPassword);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

// 配置认证相关的路由
pub fn configure_auth_routes() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("Auth Routes", |rocket| async {
        rocket.mount("/auth", routes![register, login])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::{ContentType, Status};
    use rocket::local::asynchronous::Client;
    use rocket::State;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_register_success() {
        // 初始化 Rocket 实例并启动测试客户端
        let rb = Arc::new(Mutex::new(RBatis::new()));
        let rocket = rocket::build().manage(rb).mount("/", routes![register]);
        let client = Client::tracked(rocket)
            .await
            .expect("valid rocket instance");

        // 模拟注册请求
        let register_request = r#"{"username": "test_user", "password": "test_pass"}"#;
        let response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(register_request)
            .dispatch()
            .await;

        // 验证响应
        assert_eq!(response.status(), Status::Ok);
        let response_body = response.into_string().await.unwrap();
        assert!(response_body.contains("address"));
        assert!(response_body.contains("privatekey"));
    }

    #[tokio::test]
    async fn test_register_failure() {
        // 初始化 Rocket 实例并启动测试客户端
        let rb = Arc::new(Mutex::new(RBatis::new()));
        let rocket = rocket::build().manage(rb).mount("/", routes![register]);
        let client = Client::tracked(rocket)
            .await
            .expect("valid rocket instance");

        // 模拟注册请求，使用错误的输入触发失败
        let register_request = r#"{"username": "", "password": ""}"#;
        let response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(register_request)
            .dispatch()
            .await;

        // 验证响应
        assert_eq!(response.status(), Status::Ok);
        let response_body = response.into_string().await.unwrap();
        assert!(response_body.contains("Error"));
    }
}
#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(RBatis::new()) // 将数据库连接池加入到Rocket的状态管理中
        .mount("/", routes![register, login]) // 将注册路由加入到Rocket应用
}
