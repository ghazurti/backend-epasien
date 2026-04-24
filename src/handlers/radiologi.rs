use crate::handlers::auth::Claims;
use crate::models::radiologi::{RadiologyOrder, RadiologyResult};
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use sqlx::MySqlPool;

pub async fn get_radiology_results_list(
    State(pool): State<MySqlPool>,
    claims: Claims,
) -> impl IntoResponse {
    let no_rm = claims.sub;

    let result = sqlx::query_as::<_, RadiologyOrder>(
        "SELECT 
            pr.no_rawat,
            pr.tgl_periksa,
            TIME_FORMAT(pr.jam, '%H:%i:%s') as jam,
            d.nm_dokter,
            j.nm_perawatan,
            pr.status
         FROM periksa_radiologi pr
         INNER JOIN reg_periksa r ON pr.no_rawat = r.no_rawat
         INNER JOIN dokter d ON pr.kd_dokter = d.kd_dokter
         INNER JOIN jns_perawatan_radiologi j ON pr.kd_jenis_prw = j.kd_jenis_prw
         WHERE r.no_rkm_medis = ?
         ORDER BY pr.tgl_periksa DESC, pr.jam DESC",
    )
    .bind(no_rm)
    .fetch_all(&pool)
    .await;

    match result {
        Ok(orders) => Json(json!({
            "status": "success",
            "data": orders
        })),
        Err(e) => {
            eprintln!("[radiologi] DB error list: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}

pub async fn get_radiology_result_detail(
    axum::extract::Path(no_rawat): axum::extract::Path<String>,
    State(pool): State<MySqlPool>,
    claims: Claims,
) -> impl IntoResponse {
    let no_rm = claims.sub;

    // Verify this no_rawat belongs to the patient
    let verify =
        sqlx::query("SELECT no_rawat FROM reg_periksa WHERE no_rawat = ? AND no_rkm_medis = ?")
            .bind(&no_rawat)
            .bind(&no_rm)
            .fetch_optional(&pool)
            .await;

    match verify {
        Ok(None) => {
            return Json(json!({
                "status": "error",
                "message": "Data radiologi tidak ditemukan"
            }));
        }
        Err(e) => {
            eprintln!("[radiologi] DB error verify: {}", e);
            return Json(json!({"status": "error", "message": "Terjadi kesalahan server"}));
        }
        _ => {}
    }

    // Get radiology results (clinical report)
    let result = sqlx::query_as::<_, RadiologyResult>(
        "SELECT 
            hr.no_rawat,
            hr.tgl_periksa,
            TIME_FORMAT(hr.jam, '%H:%i:%s') as jam,
            hr.hasil
         FROM hasil_radiologi hr
         WHERE hr.no_rawat = ?",
    )
    .bind(&no_rawat)
    .fetch_all(&pool)
    .await;

    match result {
        Ok(results) => Json(json!({
            "status": "success",
            "data": results
        })),
        Err(e) => {
            eprintln!("[radiologi] DB error detail: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}
