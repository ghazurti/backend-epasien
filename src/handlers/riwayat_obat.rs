use crate::handlers::auth::Claims;
use axum::{extract::State, Json};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::{FromRow, MySqlPool};

#[derive(Debug, FromRow)]
struct ResepRow {
    no_resep: String,
    tgl_peresepan: Option<chrono::NaiveDate>,
    nm_dokter: String,
    nama_obat: String,
    jml: Option<f64>,
    aturan_pakai: String,
}

#[derive(Debug, Serialize)]
struct ObatItem {
    nama_obat: String,
    jml: f64,
    aturan_pakai: String,
}

#[derive(Debug, Serialize)]
struct Resep {
    no_resep: String,
    tgl_peresepan: Option<chrono::NaiveDate>,
    nm_dokter: String,
    obat: Vec<ObatItem>,
}

pub async fn get_riwayat_obat(
    State(pool): State<MySqlPool>,
    claims: Claims,
) -> Json<Value> {
    let rows = sqlx::query_as::<_, ResepRow>(
        "SELECT ro.no_resep, ro.tgl_peresepan, \
         COALESCE(dk.nm_dokter, '') AS nm_dokter, \
         COALESCE(db2.nama_brng, '') AS nama_obat, \
         rd.jml, COALESCE(rd.aturan_pakai, '') AS aturan_pakai \
         FROM resep_obat ro \
         JOIN resep_dokter rd ON ro.no_resep = rd.no_resep \
         JOIN reg_periksa rp ON ro.no_rawat = rp.no_rawat \
         LEFT JOIN dokter dk ON ro.kd_dokter = dk.kd_dokter \
         LEFT JOIN databarang db2 ON rd.kode_brng = db2.kode_brng \
         WHERE rp.no_rkm_medis = ? \
         ORDER BY ro.tgl_peresepan DESC, ro.no_resep, db2.nama_brng",
    )
    .bind(&claims.sub)
    .fetch_all(&pool)
    .await;

    match rows {
        Ok(rows) => {
            let mut resep_list: Vec<Resep> = Vec::new();
            for row in rows {
                let obat_item = ObatItem {
                    nama_obat: row.nama_obat,
                    jml: row.jml.unwrap_or(0.0),
                    aturan_pakai: row.aturan_pakai,
                };
                if let Some(last) = resep_list.last_mut() {
                    if last.no_resep == row.no_resep {
                        last.obat.push(obat_item);
                        continue;
                    }
                }
                resep_list.push(Resep {
                    no_resep: row.no_resep,
                    tgl_peresepan: row.tgl_peresepan,
                    nm_dokter: row.nm_dokter,
                    obat: vec![obat_item],
                });
            }
            Json(json!({"status": "success", "data": resep_list}))
        }
        Err(e) => {
            eprintln!("[riwayat_obat::get_riwayat_obat] DB error: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}
