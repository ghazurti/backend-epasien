use crate::fcm::SharedFcmClient;
use crate::handlers::auth::Claims;
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::MySqlPool;

#[derive(Deserialize)]
pub struct RegisterTokenRequest {
    pub token: String,
    pub platform: Option<String>,
}

pub async fn register_token(
    State(pool): State<MySqlPool>,
    claims: Claims,
    Json(payload): Json<RegisterTokenRequest>,
) -> Json<Value> {
    let platform = payload.platform.unwrap_or_else(|| "android".to_string());

    let result = sqlx::query(
        "INSERT INTO fcm_tokens (no_rkm_medis, token, platform)
         VALUES (?, ?, ?)
         ON DUPLICATE KEY UPDATE token = VALUES(token), platform = VALUES(platform), updated_at = NOW()",
    )
    .bind(&claims.sub)
    .bind(&payload.token)
    .bind(&platform)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Json(json!({"status": "success", "message": "Token terdaftar"})),
        Err(e) => {
            eprintln!("[notification::register_token] DB error: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}

// Utility: kirim notif ke satu pasien berdasarkan no_rkm_medis
pub async fn send_to_pasien(
    pool: &MySqlPool,
    fcm: &SharedFcmClient,
    no_rkm_medis: &str,
    title: &str,
    body: &str,
    data: Option<serde_json::Value>,
) {
    let token_row = sqlx::query_scalar::<_, String>(
        "SELECT token FROM fcm_tokens WHERE no_rkm_medis = ?",
    )
    .bind(no_rkm_medis)
    .fetch_optional(pool)
    .await;

    match token_row {
        Ok(Some(token)) => {
            let mut client = fcm.lock().await;
            if let Err(e) = client.send(&token, title, body, data).await {
                eprintln!("[fcm] Gagal kirim notif ke {}: {}", no_rkm_medis, e);
            }
        }
        Ok(None) => {}
        Err(e) => eprintln!("[fcm] DB error fetch token: {}", e),
    }
}
