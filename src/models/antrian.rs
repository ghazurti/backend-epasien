use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AntrianRow {
    pub no_reg: Option<String>,
    pub kd_poli: Option<String>,
    pub nm_poli: String,
    pub kd_dokter: Option<String>,
    pub nm_dokter: String,
    pub jam_reg: Option<chrono::NaiveTime>,
    pub antrian_didepan: i64,
    pub total_pasien: i64,
}
