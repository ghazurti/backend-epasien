use crate::handlers::auth::Claims;
use crate::models::lab_results::{LabOrder, LabResult};
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use sqlx::MySqlPool;

pub async fn get_lab_results_list(
    State(pool): State<MySqlPool>,
    claims: Claims,
) -> impl IntoResponse {
    let no_rm = claims.sub;

    let result = sqlx::query_as::<_, LabOrder>(
        "SELECT 
            pl.no_rawat,
            pl.tgl_periksa,
            TIME_FORMAT(pl.jam, '%H:%i:%s') as jam,
            d.nm_dokter,
            p.nm_poli,
            pl.status
         FROM periksa_lab pl
         INNER JOIN reg_periksa r ON pl.no_rawat = r.no_rawat
         INNER JOIN dokter d ON pl.kd_dokter = d.kd_dokter
         INNER JOIN poliklinik p ON r.kd_poli = p.kd_poli
         WHERE r.no_rkm_medis = ?
         ORDER BY pl.tgl_periksa DESC, pl.jam DESC",
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
            eprintln!("[lab] DB error list: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}

pub async fn get_lab_result_detail(
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
                "message": "Data lab tidak ditemukan"
            }));
        }
        Err(e) => {
            eprintln!("[lab] DB error verify: {}", e);
            return Json(json!({"status": "error", "message": "Terjadi kesalahan server"}));
        }
        _ => {}
    }

    // Get lab results
    let result = sqlx::query_as::<_, LabResult>(
        "SELECT
            dpl.no_rawat,
            pl.tgl_periksa,
            TIME_FORMAT(pl.jam, '%H:%i:%s') as jam,
            COALESCE(dk.nm_dokter, '') as nm_dokter,
            tl.Pemeriksaan as nm_perawatan,
            dpl.nilai,
            COALESCE(NULLIF(dpl.nilai_rujukan, ''), tl.nilai_rujukan_ld, '') as nilai_rujukan,
            tl.satuan,
            dpl.keterangan
         FROM detail_periksa_lab dpl
         INNER JOIN periksa_lab pl ON dpl.no_rawat = pl.no_rawat
            AND dpl.kd_jenis_prw = pl.kd_jenis_prw
            AND dpl.tgl_periksa = pl.tgl_periksa
            AND dpl.jam = pl.jam
         INNER JOIN template_laboratorium tl ON dpl.id_template = tl.id_template
         LEFT JOIN dokter dk ON pl.kd_dokter = dk.kd_dokter
         WHERE dpl.no_rawat = ?
         ORDER BY tl.urut",
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
            eprintln!("[lab] DB error detail: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}
