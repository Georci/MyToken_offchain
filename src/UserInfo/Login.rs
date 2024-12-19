use crate::DataBase::get_db;
use crate::DataBase::Users;
use crate::Error::UserError;
use crate::IdentityAuthentication::Jwt::generate_token;
use crate::UserInfo::Generate_address::generate_random_account;
use alloy::hex;
use jsonwebtoken::{decode, DecodingKey, Validation};
use jsonwebtoken::{encode, EncodingKey, Header};
/// register\login\verify
/// 查询用户信息 图片内容
use rbatis::executor::Executor;
use rbatis::Error;
use rbatis::RBatis;
use rocket::tokio;
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

// 注册用户
pub async fn register_user(
    username: &str,
    password: &str,
    company_name: &str,
) -> Result<(String, String), UserError> {
    // 获取数据库连接池
    let rb = get_db().await;

    // 检查用户是否已存在
    match Users::select_by_username(rb, username).await {
        Ok(Some(user)) => Err(UserError::UserAlreadyExists),
        Ok(None) => {
            // 为新用户生成账户（调用您已有的函数 `generate_random_account`）
            let (private_key, address) = generate_random_account();

            // 哈希用户密码（用于安全存储）
            let mut hasher = Sha256::new();
            hasher.update(password);
            let password_hash = hex::encode(hasher.finalize());
            // 插入用户数据
            let table = Users {
                id: None,
                company_name: Some(company_name.to_string()),
                username: Some(username.to_string()),
                password: Some(password_hash.to_string()),
                watermark_base64: Some("".to_string()),
                address: Some(format!("0x{}", hex::encode(address)).to_string()),
                privatekey: Some(format!("0x{}", hex::encode(private_key)).to_string()),
            };
            let data = Users::insert(rb, &table)
                .await
                .map_err(|_| UserError::DatabaseError(Error::E("insert data error".into())))?;
            println!("insert = {}", json!(data));
            // 返回以太坊地址
            Ok((
                format!("0x{}", hex::encode(address)),
                format!("0x{}", hex::encode(private_key)),
            ))
        }
        Err(e) => Err(UserError::DatabaseError(Error::E(
            "Database query error".into(),
        ))),
    }
}

pub async fn login_user(username: &str, password: &str) -> Result<(String, String), UserError> {
    // 获取数据库连接池
    let rb = get_db().await;

    // 获取用户数据
    match Users::select_by_username(rb, username).await {
        Ok(Some(user)) => {
            // 验证用户密码
            if let Some(stored_password_hash) = user.password {
                // 使用相同的哈希算法计算输入密码的哈希值
                let mut hasher = Sha256::new();
                hasher.update(password);
                let input_password_hash = hex::encode(hasher.finalize());

                if input_password_hash == stored_password_hash {
                    // 密码匹配，返回以太坊地址
                    if let Some(address) = user.address {
                        let token = generate_token(username);
                        Ok((address, token)) // 返回用户的以太坊地址
                    } else {
                        Err(UserError::DatabaseError(Error::E(
                            "Database query error".into(),
                        )))
                    }
                } else {
                    // 密码不匹配
                    Err(UserError::InvalidPassword)
                }
            } else {
                Err(UserError::InvalidPassword)
            }
        }
        Ok(None) => {
            println!("User not found");
            Err(UserError::UserNotFound)
        }
        Err(e) => {
            eprintln!("Error querying user: {:?}", e);
            Err(UserError::DatabaseError(Error::E(
                "Database query error".into(),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_user() {
        let (userAddress, userPk) = register_user("Krooosss", "123", "Gzhu").await.unwrap();
    }

    #[tokio::test]
    async fn test_login_user() {
        let result = login_user("kenijima", "123").await.unwrap();
        println!("result :{:?}", result);
    }
}
