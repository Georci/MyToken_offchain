use crate::IdentityAuthentication::Jwt::validate_token;
use crate::Router::User_routers::*;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{fairing::AdHoc, Build, Request, Rocket, Route};

// 受保护资源路由
#[get("/protected")]
pub async fn protected_resource() -> &'static str {
    "You have accessed a protected resource!"
}

// 配置受保护资源的路由
pub fn configure_protected_routes() -> AdHoc {
    AdHoc::on_ignite("Protected Routes", |rocket| async {
        rocket.mount("/protected", routes![protected_resource])
    })
}

pub struct AuthenticatedUser {
    pub username: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(auth_header) = request.headers().get_one("Authorization") {
            let token = auth_header.trim_start_matches("Bearer ");
            match validate_token(token) {
                Ok(claims) => Outcome::Success(AuthenticatedUser {
                    username: claims.sub,
                }),
                Err(_) => Outcome::Error((Status::Unauthorized, ())),
            }
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}
