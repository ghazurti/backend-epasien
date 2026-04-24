use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Booking {
    pub tanggal_booking: Option<chrono::NaiveDate>,
    pub jam_booking: Option<chrono::NaiveTime>,
    pub no_rkm_medis: String,
    pub tanggal_periksa: chrono::NaiveDate,
    pub kd_dokter: Option<String>,
    pub kd_poli: Option<String>,
    pub no_reg: Option<String>,
    pub kd_pj: Option<String>,
    pub limit_reg: Option<i32>,
    pub waktu_kunjungan: Option<chrono::NaiveDateTime>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookingHistory {
    pub tanggal_booking: Option<chrono::NaiveDate>,
    pub jam_booking: Option<chrono::NaiveTime>,
    pub no_rkm_medis: String,
    pub tanggal_periksa: chrono::NaiveDate,
    pub kd_dokter: Option<String>,
    pub nm_dokter: Option<String>,
    pub kd_poli: Option<String>,
    pub nm_poli: Option<String>,
    pub no_reg: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookingRequest {
    pub no_rkm_medis: String,
    pub tanggal_periksa: String, // YYYY-MM-DD
    pub kd_dokter: String,
    pub kd_poli: String,
    pub kd_pj: String,
}

#[derive(Debug, Deserialize)]
pub struct CheckInRequest {
    pub no_rkm_medis: String,
    pub tanggal_periksa: String,
    pub kd_dokter: String,
    pub kd_poli: String,
}

#[derive(Debug, Deserialize)]
pub struct CancelBookingRequest {
    pub no_rkm_medis: String,
    pub tanggal_periksa: String,
    pub kd_dokter: String,
    pub kd_poli: String,
}
