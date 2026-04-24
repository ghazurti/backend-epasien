use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Jadwal {
    pub kd_dokter: String,
    pub nm_dokter: String,
    pub kd_poli: Option<String>,
    pub nm_poli: Option<String>,
    pub hari_kerja: String,
    pub jam_mulai: String,
    pub jam_selesai: Option<String>,
    pub kuota: Option<i32>,
}
