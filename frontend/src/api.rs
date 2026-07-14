use gloo_net::http::{Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use shared::ApiError;

#[derive(Debug, Clone)]
pub enum ApiClientError {
    Status(u16, String),
    CaptchaRequired,
    Network(String),
}

impl std::fmt::Display for ApiClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiClientError::Status(code, msg) => write!(f, "Lỗi {code}: {msg}"),
            ApiClientError::CaptchaRequired => write!(f, "Cần xác minh captcha"),
            ApiClientError::Network(msg) => write!(f, "Lỗi kết nối: {msg}"),
        }
    }
}

fn net_err<E: std::fmt::Display>(e: E) -> ApiClientError {
    ApiClientError::Network(e.to_string())
}

async fn handle_response<T: DeserializeOwned>(resp: Response) -> Result<T, ApiClientError> {
    if resp.status() == 429 {
        return Err(ApiClientError::CaptchaRequired);
    }
    if !resp.ok() {
        let status = resp.status();
        let msg = resp
            .json::<ApiError>()
            .await
            .map(|e| e.error)
            .unwrap_or_else(|_| format!("HTTP {status}"));
        return Err(ApiClientError::Status(status, msg));
    }
    resp.json::<T>().await.map_err(net_err)
}

async fn handle_empty_response(resp: Response) -> Result<(), ApiClientError> {
    if resp.status() == 429 {
        return Err(ApiClientError::CaptchaRequired);
    }
    if !resp.ok() {
        let status = resp.status();
        let msg = resp
            .json::<ApiError>()
            .await
            .map(|e| e.error)
            .unwrap_or_else(|_| format!("HTTP {status}"));
        return Err(ApiClientError::Status(status, msg));
    }
    Ok(())
}

pub async fn get_json<T: DeserializeOwned>(path: &str) -> Result<T, ApiClientError> {
    let resp = Request::get(path).send().await.map_err(net_err)?;
    handle_response(resp).await
}

pub async fn post_json<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, ApiClientError> {
    let resp = Request::post(path)
        .json(body)
        .map_err(net_err)?
        .send()
        .await
        .map_err(net_err)?;
    handle_response(resp).await
}

pub async fn put_json<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, ApiClientError> {
    let resp = Request::put(path)
        .json(body)
        .map_err(net_err)?
        .send()
        .await
        .map_err(net_err)?;
    handle_response(resp).await
}

pub async fn post_empty(path: &str) -> Result<(), ApiClientError> {
    let resp = Request::post(path).send().await.map_err(net_err)?;
    handle_empty_response(resp).await
}

pub async fn delete_empty(path: &str) -> Result<(), ApiClientError> {
    let resp = Request::delete(path).send().await.map_err(net_err)?;
    handle_empty_response(resp).await
}
