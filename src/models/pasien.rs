use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Pasien {
    pub no_rkm_medis: String,
    pub nm_pasien: String,
    pub alamat: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub no_rm: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub no_rm: String,
    pub old_password: String,
    pub new_password: String,
}
