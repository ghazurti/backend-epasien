use crate::handlers::auth::Claims;
use crate::models::antrian::AntrianRow;
use axum::{extract::State, Json};
use serde_json::{json, Value};
use sqlx::MySqlPool;

pub async fn get_antrian_status(
    State(pool): State<MySqlPool>,
    claims: Claims,
) -> Json<Value> {
    let result = sqlx::query_as::<_, AntrianRow>(
        "SELECT \
            rp.no_reg, \
            rp.kd_poli, \
            COALESCE(pl.nm_poli, '') AS nm_poli, \
            rp.kd_dokter, \
            COALESCE(dk.nm_dokter, '') AS nm_dokter, \
            rp.jam_reg, \
            (SELECT COUNT(*) \
             FROM reg_periksa r2 \
             WHERE r2.kd_poli = rp.kd_poli \
               AND r2.kd_dokter = rp.kd_dokter \
               AND DATE(r2.tgl_registrasi) = CURDATE() \
               AND r2.stts != 'Batal' \
               AND CAST(r2.no_reg AS UNSIGNED) < CAST(rp.no_reg AS UNSIGNED) \
            ) AS antrian_didepan, \
            (SELECT COUNT(*) \
             FROM reg_periksa r3 \
             WHERE r3.kd_poli = rp.kd_poli \
               AND r3.kd_dokter = rp.kd_dokter \
               AND DATE(r3.tgl_registrasi) = CURDATE() \
               AND r3.stts != 'Batal' \
            ) AS total_pasien \
         FROM reg_periksa rp \
         LEFT JOIN dokter dk ON rp.kd_dokter = dk.kd_dokter \
         LEFT JOIN poliklinik pl ON rp.kd_poli = pl.kd_poli \
         WHERE rp.no_rkm_medis = ? \
           AND DATE(rp.tgl_registrasi) = CURDATE() \
           AND rp.stts NOT IN ('Batal', 'Sudah', 'Dirujuk', 'Meninggal', 'Dirawat', 'Pulang Paksa') \
         ORDER BY rp.jam_reg ASC \
         LIMIT 1",
    )
    .bind(&claims.sub)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(antrian)) => {
            let estimasi_menit = antrian.antrian_didepan * 15;
            Json(json!({
                "status": "success",
                "data": {
                    "no_antrian": antrian.no_reg,
                    "kd_poli": antrian.kd_poli,
                    "nm_poli": antrian.nm_poli,
                    "kd_dokter": antrian.kd_dokter,
                    "nm_dokter": antrian.nm_dokter,
                    "jam_reg": antrian.jam_reg,
                    "antrian_didepan": antrian.antrian_didepan,
                    "total_pasien": antrian.total_pasien,
                    "estimasi_menit": estimasi_menit
                }
            }))
        }
        Ok(None) => Json(json!({
            "status": "success",
            "data": null,
            "message": "Tidak ada antrian hari ini"
        })),
        Err(e) => {
            eprintln!("[antrian::get_antrian_status] DB error: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}
