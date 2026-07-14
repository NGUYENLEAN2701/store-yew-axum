use axum::{
    async_trait, extract::FromRequestParts, http::request::Parts, http::StatusCode,
    RequestPartsExt,
};
use axum_extra::extract::CookieJar;

use crate::{security::jwt, AppState};

pub const AUTH_COOKIE: &str = "greeniem_admin_token";

/// Extractor that gates a handler behind a valid admin JWT cookie.
/// Adding `_admin: AdminUser` as a handler parameter is enough to protect a route.
pub struct AdminUser {
    pub username: String,
}

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar: CookieJar = parts
            .extract()
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "missing cookies"))?;
        let token = jar
            .get(AUTH_COOKIE)
            .map(|c| c.value().to_string())
            .ok_or((StatusCode::UNAUTHORIZED, "not authenticated"))?;
        let claims = jwt::verify_token(&token, &state.config.jwt_secret)
            .ok_or((StatusCode::UNAUTHORIZED, "invalid or expired session"))?;
        Ok(AdminUser {
            username: claims.sub,
        })
    }
}
