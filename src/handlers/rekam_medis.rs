use crate::handlers::auth::Claims;
use crate::models::rekam_medis::RekamMedis;
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use sqlx::MySqlPool;

pub async fn get_rekam_medis_history(
    State(pool): State<MySqlPool>,
    claims: Claims,
) -> impl IntoResponse {
    let no_rm = claims.sub;

    let result = sqlx::query_as::<_, RekamMedis>(
        "SELECT 
            r.no_rawat,
            r.tgl_registrasi,
            d.nm_dokter,
            p.nm_poli,
            pr.suhu_tubuh,
            pr.tensi,
            pr.nadi,
            pr.respirasi,
            pr.tinggi,
            pr.berat,
            pr.keluhan,
            pr.pemeriksaan,
            pr.penilaian,
            pr.rtl,
            pr.instruksi
         FROM reg_periksa r
         INNER JOIN dokter d ON r.kd_dokter = d.kd_dokter
         INNER JOIN poliklinik p ON r.kd_poli = p.kd_poli
         LEFT JOIN (
             SELECT 
                 no_rawat,
                 suhu_tubuh,
                 tensi,
                 nadi,
                 respirasi,
                 tinggi,
                 berat,
                 keluhan,
                 pemeriksaan,
                 penilaian,
                 rtl,
                 instruksi
             FROM pemeriksaan_ralan
             WHERE (no_rawat, tgl_perawatan, jam_rawat) IN (
                 SELECT no_rawat, MAX(tgl_perawatan), MAX(jam_rawat)
                 FROM pemeriksaan_ralan
                 GROUP BY no_rawat
             )
         ) pr ON r.no_rawat = pr.no_rawat
         WHERE r.no_rkm_medis = ?
         ORDER BY r.tgl_registrasi DESC, r.jam_reg DESC",
    )
    .bind(no_rm)
    .fetch_all(&pool)
    .await;

    match result {
        Ok(history) => Json(json!({
            "status": "success",
            "data": history
        })),
        Err(e) => {
            eprintln!("[rekam_medis] DB error: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}
