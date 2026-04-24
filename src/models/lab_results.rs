use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LabOrder {
    pub no_rawat: String,
    pub tgl_periksa: chrono::NaiveDate,
    pub jam: String,
    pub nm_dokter: String,
    pub nm_poli: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LabResult {
    pub no_rawat: String,
    pub tgl_periksa: chrono::NaiveDate,
    pub jam: String,
    pub nm_dokter: String,
    pub nm_perawatan: String,
    pub nilai: String,
    pub nilai_rujukan: String,
    pub satuan: String,
    pub keterangan: Option<String>,
}
