use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use bson::doc;
use shared::{AdminCredentials, AdminMe, SetupStatus};
use time::Duration as CookieDuration;

use crate::{
    error::AppError,
    models::admin::AdminDoc,
    security::{admin_extractor::AdminUser, admin_extractor::AUTH_COOKIE, jwt, password},
    AppState,
};

fn validate_credentials(input: &AdminCredentials) -> Result<(), AppError> {
    if input.username.trim().len() < 3 || input.username.len() > 64 {
        return Err(AppError::bad_request("username must be 3-64 characters"));
    }
    if input.password.len() < 8 || input.password.len() > 200 {
        return Err(AppError::bad_request("password must be at least 8 characters"));
    }
    Ok(())
}

fn issue_auth_cookie(jar: CookieJar, username: &str, state: &AppState) -> CookieJar {
    let token = jwt::create_token(username, &state.config.jwt_secret);
    let mut cookie = Cookie::new(AUTH_COOKIE, token);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    cookie.set_secure(state.config.cookie_secure);
    cookie.set_max_age(CookieDuration::hours(12));
    jar.add(cookie)
}

pub async fn setup_status(State(state): State<AppState>) -> Result<Json<SetupStatus>, AppError> {
    let coll = state.db.collection::<AdminDoc>("admins");
    let count = coll.count_documents(doc! {}).await?;
    Ok(Json(SetupStatus {
        needs_setup: count == 0,
    }))
}

/// Creates the very first admin account. Only works while no admin exists yet;
/// once one is created this permanently 403s, so it can't be replayed later.
pub async fn setup_admin(
    State(state): State<AppState>,
    Json(input): Json<AdminCredentials>,
) -> Result<(CookieJar, Json<AdminMe>), AppError> {
    validate_credentials(&input)?;
    let coll = state.db.collection::<AdminDoc>("admins");
    let count = coll.count_documents(doc! {}).await?;
    if count > 0 {
        return Err(AppError::forbidden("an admin account already exists"));
    }
    let password_hash = password::hash_password(&input.password).map_err(AppError::internal)?;
    let doc = AdminDoc {
        id: None,
        username: input.username.clone(),
        password_hash,
    };
    coll.insert_one(&doc).await?;

    let jar = issue_auth_cookie(CookieJar::new(), &input.username, &state);
    Ok((
        jar,
        Json(AdminMe {
            username: input.username,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<AdminCredentials>,
) -> Result<(CookieJar, Json<AdminMe>), AppError> {
    let coll = state.db.collection::<AdminDoc>("admins");
    let admin = coll
        .find_one(doc! { "username": &input.username })
        .await?
        .ok_or_else(|| AppError::unauthorized("invalid username or password"))?;
    if !password::verify_password(&input.password, &admin.password_hash) {
        return Err(AppError::unauthorized("invalid username or password"));
    }
    let jar = issue_auth_cookie(CookieJar::new(), &admin.username, &state);
    Ok((
        jar,
        Json(AdminMe {
            username: admin.username,
        }),
    ))
}

pub async fn logout() -> (CookieJar, StatusCode) {
    let mut cookie = Cookie::new(AUTH_COOKIE, "");
    cookie.set_path("/");
    cookie.set_max_age(CookieDuration::seconds(0));
    (CookieJar::new().add(cookie), StatusCode::NO_CONTENT)
}

pub async fn me(admin: AdminUser) -> Json<AdminMe> {
    Json(AdminMe {
        username: admin.username,
    })
}
