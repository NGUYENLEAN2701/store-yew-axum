use std::{
    collections::VecDeque,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use dashmap::DashMap;
use rand::Rng;
use shared::{ApiError, CaptchaAnswer, CaptchaChallenge};

use crate::AppState;

const WINDOW: Duration = Duration::from_secs(10);
const MAX_REQUESTS_PER_WINDOW: usize = 10;
const VERIFIED_TTL: Duration = Duration::from_secs(5 * 60);
const CHALLENGE_TTL: Duration = Duration::from_secs(2 * 60);
const IDLE_ENTRY_TTL: Duration = Duration::from_secs(10 * 60);

#[derive(Default)]
struct IpState {
    hits: VecDeque<Instant>,
    verified_until: Option<Instant>,
}

/// Tracks recent request bursts per client IP and gates them behind a simple
/// server-generated math captcha once they exceed the allowed rate.
#[derive(Clone)]
pub struct CaptchaState {
    ip_states: Arc<DashMap<IpAddr, IpState>>,
    challenges: Arc<DashMap<String, (i64, Instant)>>,
}

impl CaptchaState {
    pub fn new() -> Self {
        let state = Self {
            ip_states: Arc::new(DashMap::new()),
            challenges: Arc::new(DashMap::new()),
        };
        state.spawn_cleanup_task();
        state
    }

    fn spawn_cleanup_task(&self) {
        let ip_states = self.ip_states.clone();
        let challenges = self.challenges.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                let now = Instant::now();
                ip_states.retain(|_, s| {
                    let recently_active = s
                        .hits
                        .back()
                        .map(|t| now.duration_since(*t) < IDLE_ENTRY_TTL)
                        .unwrap_or(false);
                    let still_verified = s.verified_until.map(|v| v > now).unwrap_or(false);
                    recently_active || still_verified
                });
                challenges.retain(|_, (_, created)| now.duration_since(*created) < CHALLENGE_TTL);
            }
        });
    }

    /// Returns `true` if the request is allowed through, `false` if this IP just
    /// tripped the rate limit and must solve a captcha before continuing.
    pub fn check(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut entry = self.ip_states.entry(ip).or_default();

        if let Some(verified_until) = entry.verified_until {
            if verified_until > now {
                return true;
            }
        }

        while let Some(front) = entry.hits.front() {
            if now.duration_since(*front) > WINDOW {
                entry.hits.pop_front();
            } else {
                break;
            }
        }
        entry.hits.push_back(now);
        entry.hits.len() <= MAX_REQUESTS_PER_WINDOW
    }

    pub fn new_challenge(&self) -> CaptchaChallenge {
        let (a, b, token): (i64, i64, String) = {
            let mut rng = rand::thread_rng();
            let a = rng.gen_range(1..20);
            let b = rng.gen_range(1..20);
            let token_bytes: [u8; 16] = rng.gen();
            let token = token_bytes.iter().map(|byte| format!("{byte:02x}")).collect();
            (a, b, token)
        };
        self.challenges.insert(token.clone(), (a + b, Instant::now()));
        CaptchaChallenge {
            token,
            question: format!("{a} + {b} = ?"),
        }
    }

    pub fn verify(&self, ip: IpAddr, answer: &CaptchaAnswer) -> bool {
        if let Some((_, (expected, _))) = self.challenges.remove(&answer.token) {
            if answer.answer.trim().parse::<i64>().ok() == Some(expected) {
                let mut entry = self.ip_states.entry(ip).or_default();
                entry.verified_until = Some(Instant::now() + VERIFIED_TTL);
                entry.hits.clear();
                return true;
            }
        }
        false
    }
}

pub fn extract_ip(headers: &HeaderMap, fallback: SocketAddr) -> IpAddr {
    if let Some(ip) = headers
        .get("fly-client-ip")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
    {
        return ip;
    }
    if let Some(ip) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|v| v.trim().parse().ok())
    {
        return ip;
    }
    fallback.ip()
}

pub async fn rate_limit_captcha(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    if request.uri().path().starts_with("/api/captcha/") {
        return next.run(request).await;
    }

    let ip = extract_ip(request.headers(), addr);
    if state.captcha.check(ip) {
        next.run(request).await
    } else {
        (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ApiError {
                error: "too_many_requests".to_string(),
                captcha_required: true,
            }),
        )
            .into_response()
    }
}
