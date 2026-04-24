use crate::models::jadwal::Jadwal;
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use sqlx::MySqlPool;

pub async fn get_jadwal_dokter(State(pool): State<MySqlPool>) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Jadwal>(
        "SELECT j.kd_dokter, d.nm_dokter, j.kd_poli, p.nm_poli, j.hari_kerja, \
         CAST(j.jam_mulai AS CHAR) as jam_mulai, \
         CAST(j.jam_selesai AS CHAR) as jam_selesai, \
         j.kuota \
         FROM jadwal j \
         JOIN dokter d ON j.kd_dokter = d.kd_dokter \
         LEFT JOIN poliklinik p ON j.kd_poli = p.kd_poli \
         WHERE j.hari_kerja <> ''",
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(jadwal) => Json(json!({
            "status": "success",
            "message": "Data jadwal berhasil diambil",
            "data": jadwal
        })),
        Err(e) => {
            eprintln!("[jadwal] DB error: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}
