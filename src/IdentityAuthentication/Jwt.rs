use jsonwebtoken::{decode, DecodingKey, Validation};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub(crate) sub: String, // 用户名或用户 ID
    exp: usize,             // 过期时间
}

pub fn generate_token(username: &str) -> String {
    let my_claims = Claims {
        sub: username.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };
    encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret("secret_key".as_ref()),
    )
    .unwrap()
}

// 验证 Token 的函数
pub fn validate_token(token: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret("secret_key".as_ref()),
        &Validation::default(),
    )
    .map(|data| {
        println!("data.claims is{:?}", data.claims.clone());
        data.claims
    })
    .map_err(|e| e.to_string())
}
