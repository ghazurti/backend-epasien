use crate::models::pasien::{ChangePasswordRequest, LoginRequest, Pasien};
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::MySqlPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // no_rkm_medis
    pub exp: u64,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(json!({"status": "error", "message": "Authorization header tidak ditemukan"})),
            ))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"status": "error", "message": "Tipe token tidak valid"})),
            ));
        }

        let token = &auth_header[7..];
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"status": "error", "message": "Token tidak valid atau sudah kedaluwarsa"})),
            )
        })?;

        Ok(token_data.claims)
    }
}

pub async fn login_pasien(
    State(pool): State<MySqlPool>,
    Json(payload): Json<LoginRequest>,
) -> Json<Value> {
    // 0. Ambil Key dari Env
    let aes_key = std::env::var("AES_KEY").unwrap_or_else(|_| "windi".to_string());

    // 1. Cari pasien berdasarkan No RM dan Password
    // Fallback: Jika belum ada di personal_pasien, gunakan tgl_lahir sebagai password
    let result = sqlx::query_as::<_, Pasien>(
        "SELECT p.no_rkm_medis, p.nm_pasien, p.alamat \
         FROM pasien p \
         LEFT JOIN personal_pasien pp ON p.no_rkm_medis = pp.no_rkm_medis \
         WHERE p.no_rkm_medis = ? \
         AND ( \
            (pp.password = AES_ENCRYPT(?, ?)) \
            OR \
            (pp.no_rkm_medis IS NULL AND p.tgl_lahir = ?) \
         )",
    )
    .bind(&payload.no_rm)
    .bind(&payload.password)
    .bind(&aes_key)
    .bind(&payload.password) // for tgl_lahir check
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(pasien)) => {
            // Generate JWT Token
            let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
            let expiration = chrono::Utc::now()
                .checked_add_signed(chrono::Duration::days(30))
                .expect("valid timestamp")
                .timestamp() as u64;

            let claims = Claims {
                sub: pasien.no_rkm_medis.clone(),
                exp: expiration,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(jwt_secret.as_ref()),
            )
            .unwrap_or_default();

            Json(json!({
                "status": "success",
                "message": "Login berhasil",
                "token": token,
                "data": pasien
            }))
        }
        Ok(None) => Json(json!({
            "status": "error",
            "message": "Nomor Rekam Medis tidak terdaftar atau password salah"
        })),
        Err(e) => {
            eprintln!("[auth::login] DB error: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}

pub async fn change_password(
    State(pool): State<MySqlPool>,
    claims: Claims,
    Json(payload): Json<ChangePasswordRequest>,
) -> Json<Value> {
    // Pastikan hanya bisa ganti password milik sendiri
    if claims.sub != payload.no_rm {
        return Json(json!({
            "status": "error",
            "message": "Tidak dapat mengubah password pasien lain"
        }));
    }

    // 0. Ambil Key dari Env
    let aes_key = std::env::var("AES_KEY").unwrap_or_else(|_| "windi".to_string());

    // 1. Verifikasi Password Lama - cek di personal_pasien ATAU tgl_lahir
    let check = sqlx::query(
        "SELECT p.no_rkm_medis \
         FROM pasien p \
         LEFT JOIN personal_pasien pp ON p.no_rkm_medis = pp.no_rkm_medis \
         WHERE p.no_rkm_medis = ? \
         AND ( \
            (pp.password = AES_ENCRYPT(?, ?)) \
            OR \
            (pp.no_rkm_medis IS NULL AND p.tgl_lahir = ?) \
         )",
    )
    .bind(&payload.no_rm)
    .bind(&payload.old_password)
    .bind(&aes_key)
    .bind(&payload.old_password) // for tgl_lahir check
    .fetch_optional(&pool)
    .await;

    match check {
        Ok(Some(_)) => {
            // 2. Update ke Password Baru (Upsert logic to match menyimpantf)
            let update_result = sqlx::query(
                "INSERT INTO personal_pasien (no_rkm_medis, password) \
                 VALUES (?, AES_ENCRYPT(?, ?)) \
                 ON DUPLICATE KEY UPDATE password = AES_ENCRYPT(?, ?)",
            )
            .bind(&payload.no_rm)
            .bind(&payload.new_password)
            .bind(&aes_key)
            .bind(&payload.new_password) // for update part
            .bind(&aes_key) // for update part
            .execute(&pool)
            .await;

            match update_result {
                Ok(_) => Json(json!({
                    "status": "success",
                    "message": "Password berhasil diperbarui"
                })),
                Err(e) => {
                    eprintln!("[auth::change_password] DB error update: {}", e);
                    Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
                }
            }
        }
        Ok(None) => Json(json!({
            "status": "error",
            "message": "Password lama tidak sesuai"
        })),
        Err(e) => {
            eprintln!("[auth::change_password] DB error verify: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}
