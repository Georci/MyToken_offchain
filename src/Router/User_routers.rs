use crate::DataBase::get_db;
use crate::DataBase::Users;
use crate::UserInfo::Generate_address::generate_random_account;
use crate::UserInfo::Login::{login_user, register_user};
use rbatis::executor::Executor;
use rbatis::Error;
use rbatis::RBatis;
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

#[post("/register", format = "json", data = "<register_request>")]
async fn register(
    register_request: Json<RegisterRequest>,
    rb: &State<RBatis>,
) -> Json<RegisterResponse> {
    let result = register_user(&register_request.username, &register_request.password).await;
    match result {
        Ok((address, private_key)) => Json(RegisterResponse {
            address,
            privatekey: private_key,
        }),
        Err(error) => {
            eprintln!("Error: {}", error);
            // 返回错误信息
            Json(RegisterResponse {
                address: "Error".to_string(),
                privatekey: "Error".to_string(),
            })
        }
    }
}

// Web API 处理函数
#[post("/login", format = "json", data = "<login_request>")]
async fn login(
    login_request: Json<LoginRequest>,
    rb: &State<Arc<Mutex<RBatis>>>,
) -> Json<LoginResponse> {
    let result = login_user(&login_request.username, &login_request.password).await;
    match result {
        Ok((address, token)) => Json(LoginResponse {
            address: Some(address),
            token: Some(token),
        }),
        Err(error) => Json(LoginResponse {
            address: None,
            token: None,
        }),
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
    use rocket::local::asynchronous::Client;
}
#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(RBatis::new()) // 将数据库连接池加入到Rocket的状态管理中
        .mount("/", routes![register, login]) // 将注册路由加入到Rocket应用
}

// // 启动 Rocket 应用
// #[launch]
// fn rocket() -> _ {
//     rocket::build()
//         .manage(Arc::new(Mutex::new(RBatis::new())))  // 数据库连接池
//         .mount("/", routes![login])  // 注册 login 路由
// }
