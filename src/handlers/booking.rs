use crate::fcm::SharedFcmClient;
use crate::handlers::auth::Claims;
use crate::handlers::notification::send_to_pasien;
use crate::models::booking::{
    Booking, BookingHistory, BookingRequest, CancelBookingRequest, CheckInRequest,
};
use axum::{extract::Extension, extract::State, response::IntoResponse, Json};
use chrono::Local;
use serde_json::json;
use sqlx::MySqlPool;

pub async fn create_booking(
    State(pool): State<MySqlPool>,
    Extension(fcm): Extension<Option<SharedFcmClient>>,
    _claims: Claims,
    Json(payload): Json<BookingRequest>,
) -> impl IntoResponse {
    let now = Local::now();
    let tanggal_booking = now.format("%Y-%m-%d").to_string();
    let jam_booking = now.format("%H:%M:%S").to_string();

    // 0. Cek apakah pasien sudah booking di tanggal tersebut
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM booking_registrasi WHERE no_rkm_medis = ? AND tanggal_periksa = ?",
    )
    .bind(&payload.no_rkm_medis)
    .bind(&payload.tanggal_periksa)
    .fetch_one(&pool)
    .await;

    if let Ok(count) = existing {
        if count > 0 {
            return Json(json!({
                "status": "error",
                "message": "Pasien sudah terdaftar/booking untuk tanggal periksa tersebut"
            }));
        }
    }

    // 1. Hitung No Reg Otomatis (Count + 1)
    let count_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM booking_registrasi WHERE kd_dokter = ? AND kd_poli = ? AND tanggal_periksa = ?"
    )
    .bind(&payload.kd_dokter)
    .bind(&payload.kd_poli)
    .bind(&payload.tanggal_periksa)
    .fetch_one(&pool)
    .await;

    let no_reg = match count_result {
        Ok(count) => format!("{:03}", count + 1),
        Err(_) => "001".to_string(),
    };

    // 2. Insert ke booking_registrasi
    let insert_result = sqlx::query(
        "INSERT INTO booking_registrasi (tanggal_booking, jam_booking, no_rkm_medis, tanggal_periksa, kd_dokter, kd_poli, no_reg, kd_pj, limit_reg, waktu_kunjungan, status) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, NOW(), 'Belum')"
    )
    .bind(&tanggal_booking)
    .bind(&jam_booking)
    .bind(&payload.no_rkm_medis)
    .bind(&payload.tanggal_periksa)
    .bind(&payload.kd_dokter)
    .bind(&payload.kd_poli)
    .bind(&no_reg)
    .bind(&payload.kd_pj)
    .execute(&pool)
    .await;

    match insert_result {
        Ok(_) => {
            // Kirim notifikasi ke pasien
            if let Some(fcm_client) = fcm {
                let no_rm = payload.no_rkm_medis.clone();
                let tgl = payload.tanggal_periksa.clone();
                let pool_clone = pool.clone();
                tokio::spawn(async move {
                    send_to_pasien(
                        &pool_clone,
                        &fcm_client,
                        &no_rm,
                        "Booking Berhasil",
                        &format!("Jadwal periksa Anda pada {} telah terdaftar.", tgl),
                        None,
                    )
                    .await;
                });
            }
            Json(json!({
                "status": "success",
                "message": "Booking berhasil terdaftar",
                "data": {
                    "no_reg": no_reg,
                    "tanggal_booking": tanggal_booking,
                    "jam_booking": jam_booking
                }
            }))
        }
        Err(e) => {
            eprintln!("[booking::create] DB error: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}

pub async fn check_in(
    State(pool): State<MySqlPool>,
    _claims: Claims,
    Json(payload): Json<CheckInRequest>,
) -> impl IntoResponse {
    let now = Local::now();
    let today = now.format("%Y-%m-%d").to_string();

    // 1. Validasi Tanggal: Apakah hari ini adalah tanggal periksa yang dijadwalkan?
    if payload.tanggal_periksa != today {
        return Json(json!({
            "status": "error",
            "message": format!("Check-in hanya bisa dilakukan pada tanggal periksa ({})", payload.tanggal_periksa)
        }));
    }

    // 1b. Validasi Waktu: Apakah saat ini sudah melewati jam selesai poli?
    // Cari hari kerja (nama hari dalam Bahasa Indonesia)
    let day_name = match now.format("%u").to_string().as_str() {
        "1" => "SENIN",
        "2" => "SELASA",
        "3" => "RABU",
        "4" => "KAMIS",
        "5" => "JUMAT",
        "6" => "SABTU",
        "7" => "AKHAD",
        _ => "",
    };

    let jadwal_jam_selesai = sqlx::query_scalar::<_, Option<String>>(
        "SELECT CAST(jam_selesai AS CHAR) FROM jadwal WHERE kd_dokter = ? AND kd_poli = ? AND hari_kerja = ?"
    )
    .bind(&payload.kd_dokter)
    .bind(&payload.kd_poli)
    .bind(day_name)
    .fetch_optional(&pool)
    .await;

    if let Ok(Some(Some(jam_selesai))) = jadwal_jam_selesai {
        let current_time = now.format("%H:%M:%S").to_string();
        if current_time > jam_selesai {
            return Json(json!({
                "status": "error",
                "message": format!("Batas waktu check-in sudah berakhir (Jam Selesai: {})", jam_selesai)
            }));
        }
    }

    // 2. Ambil data booking untuk mendapatkan no_reg dan keasaman data
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT * FROM booking_registrasi WHERE no_rkm_medis = ? AND tanggal_periksa = ? AND kd_dokter = ? AND kd_poli = ?"
    )
    .bind(&payload.no_rkm_medis)
    .bind(&payload.tanggal_periksa)
    .bind(&payload.kd_dokter)
    .bind(&payload.kd_poli)
    .fetch_optional(&pool)
    .await;

    let booking_data = match booking {
        Ok(Some(b)) => b,
        Ok(None) => {
            return Json(json!({"status": "error", "message": "Data booking tidak ditemukan"}))
        }
        Err(e) => {
            eprintln!("[booking::checkin] DB error fetch booking: {}", e);
            return Json(json!({"status": "error", "message": "Terjadi kesalahan server"}));
        }
    };

    if booking_data.status == Some("Terdaftar".to_string()) {
        return Json(
            json!({"status": "error", "message": "Anda sudah melakukan check-in sebelumnya"}),
        );
    }

    // 3. Ambil data pasien untuk detil p_jawab dan umur
    let pasien = sqlx::query!(
        "SELECT nm_pasien, alamat, namakeluarga, alamatpj, keluarga, tgl_lahir, (YEAR(CURDATE())-YEAR(tgl_lahir)) AS umur FROM pasien WHERE no_rkm_medis = ?",
        payload.no_rkm_medis
    )
    .fetch_optional(&pool)
    .await;

    let pasien_data = match pasien {
        Ok(Some(p)) => p,
        _ => return Json(json!({"status": "error", "message": "Data pasien tidak ditemukan"})),
    };

    // 4. Generate no_rawat (YYYY/MM/DD/XXXXXX)
    let date_rawat = now.format("%Y/%m/%d/").to_string();
    let last_rawat = sqlx::query_scalar::<_, Option<String>>(
        "SELECT MAX(no_rawat) FROM reg_periksa WHERE tgl_registrasi = CURDATE()",
    )
    .fetch_one(&pool)
    .await;

    let next_no = match last_rawat {
        Ok(Some(max_r)) => {
            let parts: Vec<&str> = max_r.split('/').collect();
            if parts.len() == 4 {
                parts[3].parse::<i32>().unwrap_or(0) + 1
            } else {
                1
            }
        }
        _ => 1,
    };
    let no_rawat = format!("{}{:06}", date_rawat, next_no);

    // 5. Mulai Transaksi
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[booking::checkin] DB error begin tx: {}", e);
            return Json(json!({"status": "error", "message": "Terjadi kesalahan server"}));
        }
    };

    // 5b. Hitung No Reg Real-time dari reg_periksa (Urut per dokter/poli/hari)
    let last_no_reg = sqlx::query_scalar::<_, Option<String>>(
        "SELECT MAX(no_reg) FROM reg_periksa WHERE kd_dokter = ? AND kd_poli = ? AND tgl_registrasi = CURDATE()"
    )
    .bind(&payload.kd_dokter)
    .bind(&payload.kd_poli)
    .fetch_one(&mut *tx)
    .await;

    let next_no_reg = match last_no_reg {
        Ok(Some(max_nr)) => {
            let num = max_nr.parse::<i32>().unwrap_or(0);
            format!("{:03}", num + 1)
        }
        _ => "001".to_string(),
    };

    // 6. Cek status_daftar (Lama/Baru)
    let count_reg =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reg_periksa WHERE no_rkm_medis = ?")
            .bind(&payload.no_rkm_medis)
            .fetch_one(&pool)
            .await
            .unwrap_or(0);
    let stts_daftar = if count_reg > 0 { "Lama" } else { "Baru" };

    // 7. Cek status_poli (Lama/Baru)
    let count_poli = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM reg_periksa WHERE no_rkm_medis = ? AND kd_poli = ?",
    )
    .bind(&payload.no_rkm_medis)
    .bind(&payload.kd_poli)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    let status_poli = if count_poli > 0 { "Lama" } else { "Baru" };

    // 8. Insert ke reg_periksa
    let reg_result = sqlx::query(
        "INSERT INTO reg_periksa (no_reg, no_rawat, tgl_registrasi, jam_reg, kd_dokter, no_rkm_medis, kd_poli, p_jawab, almt_pj, hubunganpj, biaya_reg, stts, stts_daftar, status_lanjut, kd_pj, umurdaftar, sttsumur, status_bayar, status_poli) \
         VALUES (?, ?, CURDATE(), CURTIME(), ?, ?, ?, ?, ?, ?, 0, 'Belum', ?, 'Ralan', ?, ?, 'Th', 'Belum Bayar', ?)"
    )
    .bind(&next_no_reg)
    .bind(&no_rawat)
    .bind(&payload.kd_dokter)
    .bind(&payload.no_rkm_medis)
    .bind(&payload.kd_poli)
    .bind(&pasien_data.namakeluarga)
    .bind(&pasien_data.alamatpj)
    .bind(&pasien_data.keluarga)
    .bind(stts_daftar)
    .bind(booking_data.kd_pj.as_deref().unwrap_or("-"))
    .bind(pasien_data.umur.unwrap_or(0) as i32)
    .bind(status_poli)
    .execute(&mut *tx)
    .await;

    if let Err(e) = reg_result {
        eprintln!("[booking::checkin] DB error insert reg_periksa: {}", e);
        return Json(json!({"status": "error", "message": "Terjadi kesalahan server"}));
    }

    // 9. Update status dan no_reg booking_registrasi
    let update_booking = sqlx::query(
        "UPDATE booking_registrasi SET status = 'Terdaftar', no_reg = ? WHERE no_rkm_medis = ? AND tanggal_periksa = ? AND kd_dokter = ? AND kd_poli = ?"
    )
    .bind(&next_no_reg)
    .bind(&payload.no_rkm_medis)
    .bind(&payload.tanggal_periksa)
    .bind(&payload.kd_dokter)
    .bind(&payload.kd_poli)
    .execute(&mut *tx)
    .await;

    if let Err(e) = update_booking {
        eprintln!("[booking::checkin] DB error update booking: {}", e);
        return Json(json!({"status": "error", "message": "Terjadi kesalahan server"}));
    }

    // 10. Commit Transaksi
    if let Err(e) = tx.commit().await {
        eprintln!("[booking::checkin] DB error commit: {}", e);
        return Json(json!({"status": "error", "message": "Terjadi kesalahan server"}));
    }

    Json(json!({
        "status": "success",
        "message": "Check-in berhasil. Anda telah terdaftar.",
        "data": {
            "no_rawat": no_rawat,
            "no_reg": next_no_reg
        }
    }))
}

pub async fn get_booking_history(
    State(pool): State<MySqlPool>,
    claims: Claims,
) -> impl IntoResponse {
    let no_rm = claims.sub;

    let result = sqlx::query_as::<_, BookingHistory>(
        "SELECT 
            b.tanggal_booking,
            b.jam_booking,
            b.no_rkm_medis,
            b.tanggal_periksa,
            b.kd_dokter,
            d.nm_dokter,
            b.kd_poli,
            p.nm_poli,
            b.no_reg,
            b.status
         FROM booking_registrasi b
         LEFT JOIN dokter d ON b.kd_dokter = d.kd_dokter
         LEFT JOIN poliklinik p ON b.kd_poli = p.kd_poli
         WHERE b.no_rkm_medis = ? 
         ORDER BY b.tanggal_periksa DESC",
    )
    .bind(no_rm)
    .fetch_all(&pool)
    .await;

    match result {
        Ok(bookings) => Json(json!({
            "status": "success",
            "data": bookings
        })),
        Err(e) => {
            eprintln!("[booking::history] DB error: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}

pub async fn cancel_booking(
    State(pool): State<MySqlPool>,
    claims: Claims,
    Json(payload): Json<CancelBookingRequest>,
) -> impl IntoResponse {
    let no_rm = claims.sub;

    // Validate that the booking belongs to the authenticated user
    if payload.no_rkm_medis != no_rm {
        return Json(json!({
            "status": "error",
            "message": "Tidak dapat membatalkan booking orang lain"
        }));
    }

    // Delete the booking from database
    let result = sqlx::query(
        "DELETE FROM booking_registrasi 
         WHERE no_rkm_medis = ? 
         AND tanggal_periksa = ? 
         AND kd_dokter = ? 
         AND kd_poli = ?",
    )
    .bind(&payload.no_rkm_medis)
    .bind(&payload.tanggal_periksa)
    .bind(&payload.kd_dokter)
    .bind(&payload.kd_poli)
    .execute(&pool)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Json(json!({
                    "status": "success",
                    "message": "Booking berhasil dibatalkan"
                }))
            } else {
                Json(json!({
                    "status": "error",
                    "message": "Booking tidak ditemukan"
                }))
            }
        }
        Err(e) => {
            eprintln!("[booking::cancel] DB error: {}", e);
            Json(json!({"status": "error", "message": "Terjadi kesalahan server"}))
        }
    }
}
