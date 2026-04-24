use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SuratKontrol {
    pub no_surat: String,
    pub no_sep: String,
    pub tgl_surat: chrono::NaiveDate,
    pub tgl_rencana: chrono::NaiveDate,
    pub kd_dokter_bpjs: String,
    pub nm_dokter_bpjs: String,
    pub kd_poli_bpjs: String,
    pub nm_poli_bpjs: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SuratKontrolDetail {
    pub no_surat: String,
    pub no_sep: String,
    pub tgl_surat: chrono::NaiveDate,
    pub tgl_rencana: chrono::NaiveDate,
    pub kd_dokter_bpjs: String,
    pub nm_dokter_bpjs: String,
    pub kd_poli_bpjs: String,
    pub nm_poli_bpjs: String,
    // Patient info from bridging_sep
    pub no_rkm_medis: String,
    pub nama_pasien: String,
    pub no_kartu: String,
}
