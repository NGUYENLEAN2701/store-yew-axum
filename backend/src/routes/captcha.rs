use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use shared::{CaptchaAnswer, CaptchaChallenge};

use crate::{security::rate_limit_captcha::extract_ip, AppState};

pub async fn challenge(State(state): State<AppState>) -> Json<CaptchaChallenge> {
    Json(state.captcha.new_challenge())
}

pub async fn verify(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(answer): Json<CaptchaAnswer>,
) -> StatusCode {
    let ip = extract_ip(&headers, addr);
    if state.captcha.verify(ip, &answer) {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}
