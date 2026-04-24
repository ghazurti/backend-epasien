use crate::handlers::auth::Claims;
use crate::handlers::pdf_generator::generate_pdf_content;
use crate::models::surat_kontrol::{SuratKontrol, SuratKontrolDetail};
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::MySqlPool;

pub async fn get_surat_kontrol_list(
    State(pool): State<MySqlPool>,
    claims: Claims,
) -> impl IntoResponse {
    let no_rm = claims.sub;

    let result = sqlx::query_as::<_, SuratKontrol>(
        "SELECT 
            sk.no_surat,
            sk.no_sep,
            sk.tgl_surat,
            sk.tgl_rencana,
            sk.kd_dokter_bpjs,
            sk.nm_dokter_bpjs,
            sk.kd_poli_bpjs,
            sk.nm_poli_bpjs
         FROM bridging_surat_kontrol_bpjs sk
         INNER JOIN bridging_sep sep ON sk.no_sep = sep.no_sep
         WHERE sep.nomr = ?
         ORDER BY sk.tgl_surat DESC",
    )
    .bind(no_rm)
    .fetch_all(&pool)
    .await;

    match result {
        Ok(list) => Json(json!({
            "status": "success",
            "data": list
        })),
        Err(e) => {
            eprintln!("[surat_kontrol] DB error list: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}

pub async fn get_surat_kontrol_detail(
    State(pool): State<MySqlPool>,
    claims: Claims,
    Path(no_surat): Path<String>,
) -> impl IntoResponse {
    let no_rm = claims.sub;

    let result = sqlx::query_as::<_, SuratKontrolDetail>(
        "SELECT 
            sk.no_surat,
            sk.no_sep,
            sk.tgl_surat,
            sk.tgl_rencana,
            sk.kd_dokter_bpjs,
            sk.nm_dokter_bpjs,
            sk.kd_poli_bpjs,
            sk.nm_poli_bpjs,
            sep.nomr as no_rkm_medis,
            sep.nama_pasien,
            sep.peserta as no_kartu
         FROM bridging_surat_kontrol_bpjs sk
         INNER JOIN bridging_sep sep ON sk.no_sep = sep.no_sep
         WHERE sk.no_surat = ? AND sep.nomr = ?",
    )
    .bind(&no_surat)
    .bind(no_rm)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(detail)) => Json(json!({
            "status": "success",
            "data": detail
        })),
        Ok(None) => Json(json!({
            "status": "error",
            "message": "Surat kontrol tidak ditemukan"
        })),
        Err(e) => {
            eprintln!("[surat_kontrol] DB error detail: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}

pub async fn download_surat_kontrol_pdf(
    State(pool): State<MySqlPool>,
    claims: Claims,
    Path(no_surat): Path<String>,
) -> impl IntoResponse {
    let no_rm = claims.sub;

    // Fetch surat kontrol detail
    let result = sqlx::query_as::<_, SuratKontrolDetail>(
        "SELECT 
            sk.no_surat,
            sk.no_sep,
            sk.tgl_surat,
            sk.tgl_rencana,
            sk.kd_dokter_bpjs,
            sk.nm_dokter_bpjs,
            sk.kd_poli_bpjs,
            sk.nm_poli_bpjs,
            sep.nomr as no_rkm_medis,
            sep.nama_pasien,
            sep.peserta as no_kartu
         FROM bridging_surat_kontrol_bpjs sk
         INNER JOIN bridging_sep sep ON sk.no_sep = sep.no_sep
         WHERE sk.no_surat = ? AND sep.nomr = ?",
    )
    .bind(&no_surat)
    .bind(no_rm)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(detail)) => {
            // Generate simple PDF content (text-based for now)
            let pdf_content = generate_pdf_content(&detail);
            let filename = format!("attachment; filename=\"surat_kontrol_{}.pdf\"", no_surat);

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE.as_str(), "application/pdf"),
                    (header::CONTENT_DISPOSITION.as_str(), filename.as_str()),
                ],
                pdf_content,
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "message": "Surat kontrol tidak ditemukan"
            })),
        )
            .into_response(),
        Err(e) => {
            eprintln!("[surat_kontrol] DB error pdf: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "message": "Terjadi kesalahan server"})),
            )
                .into_response()
        }
    }
}
