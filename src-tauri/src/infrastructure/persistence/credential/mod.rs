pub mod remembered;

use log::error;
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;

/// 凭证信息
#[derive(Debug, Clone, Default)]
pub struct Credential {
    access_token: Option<String>,
    refresh_token: Option<String>,
}

impl Credential {
    pub fn get_access_token(&self) -> Option<String> {
        self.access_token.clone()
    }

    pub fn get_refresh_token(&self) -> Option<String> {
        self.refresh_token.clone()
    }

    // 重置token
    pub fn reset_token(&mut self) {
        self.access_token = None;
        self.refresh_token = None;
    }

    // 是否处于登录状态
    pub fn is_login(&self) -> bool {
        self.access_token.is_some()
    }
}

/// 全局凭证存储
pub(crate) static CREDENTIAL: Lazy<RwLock<Credential>> =
    Lazy::new(|| RwLock::new(Credential::default()));

/// 服务器公钥存储
pub(crate) static SERVER_PUBLIC_KEY: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));

/// 刷新锁：确保同一时间只有一个请求在刷新凭证
static REFRESH_LOCK: Lazy<Arc<Mutex<()>>> = Lazy::new(|| Arc::new(Mutex::new(())));

// ============ 凭证管理函数 ============

/// 获取凭证
pub fn get_credential() -> Credential {
    CREDENTIAL.read().unwrap().clone()
}

/// 设置访问令牌
pub fn set_access_token(token: String) {
    CREDENTIAL.write().unwrap().access_token = Some(token);
}

/// 设置刷新令牌
pub fn set_refresh_token(token: String) {
    CREDENTIAL.write().unwrap().refresh_token = Some(token);
}

/// 设置完整凭证
pub fn set_credential(access_token: String, refresh_token: String) {
    let mut cred = CREDENTIAL.write().unwrap();
    cred.access_token = Some(access_token);
    cred.refresh_token = Some(refresh_token);
}

/// 清除凭证（登出）
pub fn clear_credential() {
    CREDENTIAL.write().unwrap().reset_token();
}

/// 是否已登录
pub fn is_login() -> bool {
    CREDENTIAL.read().unwrap().is_login()
}

// ============ 服务器公钥管理函数 ============

/// 获取/刷新服务器公钥（应用启动时或公钥过期时调用）
/// 支持自动重试，最多重试 3 次
pub async fn fetch_server_public_key() -> Result<(), String> {
    use crate::infrastructure::http::client::Client;

    let public_key =
        fetch_server_public_key_from_url(Client::build_url("secret/public/key").map_err(|_| {
            error!("服务器请求路径错误");
            "初始化失败".to_string()
        })?)
        .await?;

    *SERVER_PUBLIC_KEY.write().unwrap() = Some(public_key);
    Ok(())
}

pub async fn fetch_server_public_key_for_base_url(base_url: &str) -> Result<String, String> {
    use crate::{app::context::AppContext, infrastructure::http::client::Client};

    let version = AppContext::get().config().server.version.clone();
    let url =
        Client::build_url_with_parts(base_url, &version, "secret/public/key").map_err(|_| {
            error!("服务器请求路径错误");
            "初始化失败".to_string()
        })?;

    fetch_server_public_key_from_url(url).await
}

/// 初始化服务器公钥（应用启动时调用，是 fetch_server_public_key 的别名）
#[inline]
pub async fn init_server_public_key() -> Result<(), String> {
    fetch_server_public_key().await
}

/// 获取服务器公钥（必须在初始化后调用）
pub fn get_server_public_key() -> String {
    SERVER_PUBLIC_KEY.read().unwrap().clone().expect("服务器配置未初始化")
}

/// 设置服务器公钥
pub fn set_server_public_key(public_key: String) {
    *SERVER_PUBLIC_KEY.write().unwrap() = Some(public_key);
}

/// 清除服务器公钥
pub fn clear_server_public_key() {
    *SERVER_PUBLIC_KEY.write().unwrap() = None;
}

async fn fetch_server_public_key_from_url(url: reqwest::Url) -> Result<String, String> {
    use tokio::time::{sleep, Duration};

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|_| "初始化失败".to_string())?;

    let mut last_err = None;
    for attempt in 0..3 {
        if attempt > 0 {
            sleep(Duration::from_millis(1000 * attempt as u64)).await;
        }

        match client.get(url.as_str()).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    last_err = Some("初始化失败".to_string());
                    continue;
                }

                match response.text().await {
                    Ok(public_key) => {
                        if public_key.contains("-----BEGIN RSA PUBLIC KEY-----") {
                            return Ok(public_key);
                        }

                        error!("服务器响应错误");
                        return Err("初始化失败".to_string());
                    }
                    Err(_) => {
                        last_err = Some("初始化失败".to_string());
                    }
                }
            }
            Err(_) => {
                last_err = Some("初始化失败".to_string());
            }
        }
    }

    error!("获取服务器公钥失败，已达到最大重试次数");
    Err(last_err.unwrap_or_else(|| "初始化失败".to_string()))
}

// ============ 凭证刷新函数 ============

/// 刷新用户凭证（带并发控制）
pub async fn refresh_credentials() -> Result<(), String> {
    // 使用 Box::pin 避免递归检测
    Box::pin(async {
        // 获取锁，确保同一时间只有一个请求在刷新
        let _lock = REFRESH_LOCK.lock().await;

        // 获取当前的 refresh_token
        let refresh_token = get_credential().get_refresh_token().ok_or("无刷新令牌")?;

        // 使用 AppContext 中的 HTTP 客户端发送刷新请求（避免递归）
        let ctx = crate::app::context::AppContext::get();
        let response = ctx
            .main_server_client
            .post_no_retry(
                "users/refresh-credentials",
                &serde_json::json!({
                    "refresh_token": refresh_token
                }),
            )
            .await
            .map_err(|_| "刷新凭证失败")?;

        // 检查响应状态
        if response.code != Some(1) {
            return Err("凭证刷新失败".to_string());
        }

        // 提取新凭证
        let data = response.data.ok_or("响应无数据")?;
        let access_token = data.get("access_token").and_then(|t| t.as_str()).ok_or("无访问令牌")?;
        let new_refresh_token =
            data.get("refresh_token").and_then(|t| t.as_str()).ok_or("无刷新令牌")?;

        // 更新全局凭证
        set_credential(access_token.to_string(), new_refresh_token.to_string());

        Ok(())
    })
    .await
}
