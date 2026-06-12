use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{DateTime, Utc};
use magical_merchant_core::CoreError;
use magical_merchant_core::sync::SyncResult;
use magical_merchant_core::sync::client::{RemoteFile, SyncClient};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::auth;

const EVENT_SYNC_COMPLETE: &str = "sync-complete";
const EVENT_SYNC_ERROR: &str = "sync-error";

#[derive(Debug, Clone, Serialize)]
pub struct SyncStatusInfo {
    pub is_syncing: bool,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

/// フロントが「設定へ誘導」「再試行」などを出し分けられるよう、
/// エラーを kind 付きで返す
#[derive(Debug, Clone, Serialize)]
pub struct SyncErrorInfo {
    pub kind: &'static str,
    pub message: String,
}

impl SyncErrorInfo {
    fn new(kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

fn classify_core_error(e: &CoreError) -> SyncErrorInfo {
    match e {
        CoreError::NotAuthenticated => SyncErrorInfo::new(
            "notAuthenticated",
            "The server rejected the login. Log in again from Settings.",
        ),
        CoreError::Network(m) => SyncErrorInfo::new("network", format!("Network error: {m}")),
        other => SyncErrorInfo::new("other", other.to_string()),
    }
}

pub struct AppSyncState {
    pub is_syncing: AtomicBool,
    pub last_synced_at: Mutex<Option<DateTime<Utc>>>,
    pub last_error: Mutex<Option<String>>,
}

impl Default for AppSyncState {
    fn default() -> Self {
        Self {
            is_syncing: AtomicBool::new(false),
            last_synced_at: Mutex::new(None),
            last_error: Mutex::new(None),
        }
    }
}

pub struct R2SyncClient {
    http: reqwest::Client,
    base_url: String,
    token: String,
}

impl R2SyncClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            // 末尾スラッシュがあると "//files" になり Worker 側で 400 になる
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
        }
    }
}

/// 401/404 以外の非成功ステータスをエラーにする。
/// これを怠ると失敗したアップロードを成功扱いで同期状態に記録したり、
/// エラーレスポンスのボディをノート本文としてローカルに書き込んだりしてしまう。
fn check_status(resp: reqwest::Response, context: &str) -> Result<reqwest::Response, CoreError> {
    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(CoreError::NotAuthenticated);
    }
    if !status.is_success() {
        return Err(CoreError::Sync(format!("{context}: HTTP {status}")));
    }
    Ok(resp)
}

#[derive(serde::Deserialize)]
struct ListResponse {
    files: Vec<ListEntry>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListEntry {
    key: String,
    last_modified: String,
    size: u64,
}

impl SyncClient for R2SyncClient {
    async fn list_remote(&self) -> Result<Vec<RemoteFile>, CoreError> {
        let resp = self
            .http
            .get(format!("{}/files", self.base_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let resp = check_status(resp, "list")?;

        let body: ListResponse = resp
            .json()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        body.files
            .into_iter()
            .map(|entry| {
                let last_modified: DateTime<Utc> = entry
                    .last_modified
                    .parse()
                    .map_err(|e: chrono::ParseError| CoreError::Sync(e.to_string()))?;
                Ok(RemoteFile {
                    key: entry.key,
                    last_modified,
                    size: entry.size,
                })
            })
            .collect()
    }

    async fn download(&self, key: &str) -> Result<Vec<u8>, CoreError> {
        let resp = self
            .http
            .get(format!("{}/files/{}", self.base_url, key))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(CoreError::NotFound(key.to_string()));
        }

        let resp = check_status(resp, "download")?;

        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| CoreError::Network(e.to_string()))
    }

    async fn upload(
        &self,
        key: &str,
        content: &[u8],
        last_modified: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let resp = self
            .http
            .put(format!("{}/files/{}", self.base_url, key))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-Last-Modified", last_modified.to_rfc3339())
            .body(content.to_vec())
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        check_status(resp, "upload")?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CoreError> {
        let resp = self
            .http
            .delete(format!("{}/files/{}", self.base_url, key))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        check_status(resp, "delete")?;

        Ok(())
    }
}

/// panic やキャンセルでも is_syncing を確実に false へ戻すガード
struct SyncingGuard<'a>(&'a AtomicBool);

impl Drop for SyncingGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

#[tauri::command]
pub async fn sync_start(
    handle: AppHandle,
    state: State<'_, AppSyncState>,
) -> Result<SyncResult, SyncErrorInfo> {
    if state
        .is_syncing
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(SyncErrorInfo::new("busy", "Sync already in progress"));
    }
    let _guard = SyncingGuard(&state.is_syncing);

    let result = do_sync(&handle).await;

    match &result {
        Ok(sync_result) => {
            *state.last_synced_at.lock().unwrap() = Some(Utc::now());
            *state.last_error.lock().unwrap() = None;
            let _ = handle.emit(EVENT_SYNC_COMPLETE, sync_result);
        }
        Err(err) => {
            *state.last_error.lock().unwrap() = Some(err.message.clone());
            let _ = handle.emit(EVENT_SYNC_ERROR, err);
        }
    }

    result
}

async fn do_sync(handle: &AppHandle) -> Result<SyncResult, SyncErrorInfo> {
    let base_dir = handle
        .path()
        .app_data_dir()
        .map_err(|e| SyncErrorInfo::new("other", e.to_string()))?;
    let config = auth::SyncConfig::load(&base_dir);

    if !config.is_configured() {
        return Err(SyncErrorInfo::new(
            "notConfigured",
            "Sync is not set up. Add your Workers URL in Settings.",
        ));
    }

    let token = auth::get_token()
        .map_err(|e| SyncErrorInfo::new("other", e))?
        .ok_or_else(|| {
            SyncErrorInfo::new("notAuthenticated", "Not logged in. Log in from Settings.")
        })?;
    if !auth::is_token_valid(&token) {
        return Err(SyncErrorInfo::new(
            "notAuthenticated",
            "Login expired. Log in again from Settings.",
        ));
    }

    let client = R2SyncClient::new(config.workers_url, token);
    let engine = magical_merchant_core::sync::SyncEngine::new(client, base_dir);

    engine.sync().await.map_err(|e| classify_core_error(&e))
}

#[tauri::command]
pub fn sync_status(state: State<'_, AppSyncState>) -> Result<SyncStatusInfo, String> {
    Ok(SyncStatusInfo {
        is_syncing: state.is_syncing.load(Ordering::SeqCst),
        last_synced_at: *state.last_synced_at.lock().unwrap(),
        last_error: state.last_error.lock().unwrap().clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_not_authenticated_for_settings_guidance() {
        let info = classify_core_error(&CoreError::NotAuthenticated);
        assert_eq!(info.kind, "notAuthenticated");
    }

    #[test]
    fn classify_network_errors() {
        let info = classify_core_error(&CoreError::Network("timeout".into()));
        assert_eq!(info.kind, "network");
        assert!(info.message.contains("timeout"));
    }

    #[test]
    fn classify_other_errors_keep_message() {
        let info = classify_core_error(&CoreError::Sync("boom".into()));
        assert_eq!(info.kind, "other");
        assert!(info.message.contains("boom"));
    }
}
