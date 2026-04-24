use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Kamar {
    pub nm_bangsal: String,
    pub kelas: String,
    pub total: i64,
    pub kosong: i64,
    pub isi: i64,
}
