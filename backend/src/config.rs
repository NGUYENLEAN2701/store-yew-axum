use std::env;

pub struct Config {
    pub mongodb_uri: String,
    pub database_name: String,
    pub jwt_secret: String,
    pub port: u16,
    /// Whether the admin auth cookie should require HTTPS. Set to `false` only for local
    /// plain-http development; must stay `true` in production (Fly.io serves via HTTPS).
    pub cookie_secure: bool,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            mongodb_uri: env::var("MONGODB_URI").expect("MONGODB_URI must be set"),
            database_name: env::var("DATABASE_NAME").unwrap_or_else(|_| "greeniem".to_string()),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            cookie_secure: env::var("COOKIE_SECURE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
        }
    }
}
