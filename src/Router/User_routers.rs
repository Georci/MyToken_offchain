use crate::DataBase::get_db;
use crate::DataBase::Users;
use crate::UserInfo::Generate_address::generate_random_account;
use crate::UserInfo::Login::{login_user, register_user};
use rbatis::executor::Executor;
use rbatis::Error;
use rbatis::RBatis;
use rocket::http::Status;
use rocket::response::status;
use rocket::{launch, post, routes, tokio};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
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

// 已测试接口
#[post("/register", format = "json", data = "<register_request>")]
pub async fn register(
    register_request: Json<RegisterRequest>,
    rb: &State<RBatis>,
) -> Result<Json<RegisterResponse>, status::Custom<Json<RegisterResponse>>> {
    let result = register_user(&register_request.username, &register_request.password).await;
    match result {
        Ok((address, private_key)) => Ok(Json(RegisterResponse {
            address,
            privatekey: private_key,
        })),
        Err(error) => {
            eprintln!("Error: {}", error);
            // 返回错误信息和状态码
            Err(status::Custom(
                Status::BadRequest, // 返回 400 状态码
                Json(RegisterResponse {
                    address: "Error".to_string(),
                    privatekey: "Error".to_string(),
                }),
            ))
        }
    }
}

// 已测试接口
// Web API 处理函数
#[post("/login", format = "json", data = "<login_request>")]
pub async fn login(
    login_request: Json<LoginRequest>,
    rb: &State<Arc<Mutex<RBatis>>>,
) -> Result<Json<LoginResponse>, status::Custom<Json<LoginResponse>>> {
    let result = login_user(&login_request.username, &login_request.password).await;
    match result {
        Ok((address, token)) => Ok(Json(LoginResponse {
            address: Some(address),
            token: Some(token),
        })),
        Err(error) => {
            eprintln!("Error: {}", error);
            Err(status::Custom(
                Status::BadRequest,
                Json(LoginResponse {
                    address: None,
                    token: None,
                }),
            ))
        }
    }
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
