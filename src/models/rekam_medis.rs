use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RekamMedis {
    pub no_rawat: String,
    pub tgl_registrasi: chrono::NaiveDate,
    pub nm_dokter: String,
    pub nm_poli: String,
    // SOAP Data from pemeriksaan_ralan
    pub suhu_tubuh: Option<String>,
    pub tensi: Option<String>,
    pub nadi: Option<String>,
    pub respirasi: Option<String>,
    pub tinggi: Option<String>,
    pub berat: Option<String>,
    pub keluhan: Option<String>,
    pub pemeriksaan: Option<String>,
    pub penilaian: Option<String>,
    pub rtl: Option<String>,
    pub instruksi: Option<String>,
}
