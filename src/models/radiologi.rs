use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RadiologyOrder {
    pub no_rawat: String,
    pub tgl_periksa: chrono::NaiveDate,
    pub jam: String,
    pub nm_dokter: String,
    pub nm_perawatan: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RadiologyResult {
    pub no_rawat: String,
    pub tgl_periksa: chrono::NaiveDate,
    pub jam: String,
    pub hasil: String,
}
