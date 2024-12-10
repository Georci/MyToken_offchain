use crate::IdentityAuthentication::Jwt::validate_token;
use actix_web::{Error, HttpMessage, HttpRequest};
use actix_web_httpauth::extractors::bearer::BearerAuth;

// 验证逻辑
async fn validator(auth: BearerAuth) -> Result<(), Error> {
    let token = auth.token();
    match validate_token(token) {
        Ok(claims) => {
            println!("Token is valid for user: {}", claims.sub);
            Ok(())
        }
        Err(e) => {
            println!("Token validation failed: {}", e);
            Err(actix_web::error::ErrorUnauthorized("Invalid token"))
        }
    }
}
