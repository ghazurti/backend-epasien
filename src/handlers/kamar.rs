use crate::models::kamar::Kamar;
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use sqlx::MySqlPool;

pub async fn get_ketersediaan_kamar(State(pool): State<MySqlPool>) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Kamar>(
        "SELECT bangsal.nm_bangsal, kamar.kelas, \
         count(*) as total, \
         count(if(kamar.status='KOSONG', 1, NULL)) as kosong, \
         count(if(kamar.status='ISI', 1, NULL)) as isi \
         FROM kamar \
         INNER JOIN bangsal ON kamar.kd_bangsal = bangsal.kd_bangsal \
         GROUP BY bangsal.nm_bangsal, kamar.kelas \
         ORDER BY bangsal.nm_bangsal, kamar.kelas",
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(kamar) => Json(json!({
            "status": "success",
            "message": "Data ketersediaan kamar berhasil diambil",
            "data": kamar
        })),
        Err(e) => {
            eprintln!("[kamar] DB error: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}
