use serde::Serialize;
use serde_json::json;
use tauri::AppHandle;

use crate::{
    app::context::AppContext,
    infrastructure::{
        http::client::Client,
        persistence::{
            credential::{self, remembered, set_server_public_key},
            tauri_store::{self, keys},
        },
    },
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerEndpointState {
    pub default_base_url: String,
    pub custom_base_url: Option<String>,
    pub effective_base_url: String,
    pub using_custom_base_url: bool,
}

#[tauri::command]
pub fn get_server_endpoint_state(app: AppHandle) -> Result<ServerEndpointState, String> {
    Ok(build_server_endpoint_state(&app))
}

#[tauri::command]
pub async fn set_server_base_url(
    app: AppHandle,
    base_url: String,
) -> Result<ServerEndpointState, String> {
    let current_effective_base_url = Client::effective_server_base_url();
    let normalized = normalize_server_base_url(&base_url)?;
    let default_base_url = AppContext::get().config().server.base_url.clone();
    let next_custom_base_url = if normalized == default_base_url {
        None
    } else {
        Some(normalized)
    };
    let next_effective_base_url = next_custom_base_url.as_deref().unwrap_or(&default_base_url);
    let public_key =
        credential::fetch_server_public_key_for_base_url(next_effective_base_url).await?;

    tauri_store::set_store_key(
        &app,
        keys::SERVER,
        next_custom_base_url.as_ref().map_or(serde_json::Value::Null, |value| {
            json!({
                keys::server::BASE_URL: value,
            })
        }),
    )?;
    clear_saved_auth_state_if_endpoint_changed(
        &current_effective_base_url,
        next_effective_base_url,
    );
    set_server_public_key(public_key);

    Ok(build_server_endpoint_state(&app))
}

#[tauri::command]
pub async fn reset_server_base_url(app: AppHandle) -> Result<ServerEndpointState, String> {
    let current_effective_base_url = Client::effective_server_base_url();
    let default_base_url = AppContext::get().config().server.base_url.clone();
    let public_key = credential::fetch_server_public_key_for_base_url(&default_base_url).await?;

    tauri_store::set_store_key(&app, keys::SERVER, serde_json::Value::Null)?;
    clear_saved_auth_state_if_endpoint_changed(&current_effective_base_url, &default_base_url);
    set_server_public_key(public_key);

    Ok(build_server_endpoint_state(&app))
}

fn build_server_endpoint_state(app: &AppHandle) -> ServerEndpointState {
    let default_base_url = AppContext::get().config().server.base_url.clone();
    let custom_base_url = tauri_store::get_store_key(app, keys::SERVER)
        .and_then(|raw| raw.as_object().cloned())
        .and_then(|obj| {
            obj.get(keys::server::BASE_URL)
                .or_else(|| obj.get("base_url"))
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        });

    let effective_base_url =
        custom_base_url.clone().unwrap_or_else(|| Client::effective_server_base_url());

    ServerEndpointState {
        default_base_url,
        custom_base_url: custom_base_url.clone(),
        effective_base_url,
        using_custom_base_url: custom_base_url.is_some(),
    }
}

fn normalize_server_base_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("服务端地址不能为空".to_string());
    }

    let mut url = reqwest::Url::parse(trimmed)
        .map_err(|_| "服务端地址格式无效，请输入包含 http:// 或 https:// 的地址".to_string())?;

    match url.scheme() {
        "http" | "https" => {}
        _ => return Err("服务端地址仅支持 http:// 或 https://".to_string()),
    }

    if url.host_str().is_none() {
        return Err("服务端地址必须包含有效主机名".to_string());
    }

    let normalized_path = match url.path().trim_end_matches('/') {
        "" | "/" => "/api/".to_string(),
        "/api" => "/api/".to_string(),
        path => format!("{}/", path.trim_end_matches('/')),
    };

    url.set_path(&normalized_path);
    url.set_query(None);
    url.set_fragment(None);

    Ok(url.to_string())
}

fn clear_saved_auth_state_if_endpoint_changed(
    current_effective_base_url: &str,
    next_effective_base_url: &str,
) {
    if current_effective_base_url == next_effective_base_url {
        return;
    }

    credential::clear_credential();

    if let Err(error) = remembered::clear_remembered_credential() {
        log::warn!(
            "failed to clear remembered credential after server switch: {}",
            error
        );
    }
}
