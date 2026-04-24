mod db;
mod fcm;
mod models {
    pub mod antrian;
    pub mod booking;
    pub mod jadwal;
    pub mod kamar;
    pub mod lab_results;
    pub mod pasien;
    pub mod radiologi;
    pub mod rekam_medis;
    pub mod surat_kontrol;
}
mod handlers {
    pub mod antrian;
    pub mod auth;
    pub mod booking;
    pub mod jadwal;
    pub mod kamar;
    pub mod lab_results;
    pub mod news;
    pub mod pdf_generator;
    pub mod radiologi;
    pub mod rekam_medis;
    pub mod notification;
    pub mod riwayat_obat;
    pub mod surat_kontrol;
}
mod middleware;

use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pool = db::init_pool().await;
    let fcm_client = fcm::create_fcm_client();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create rate limiter
    let rate_limiter = middleware::rate_limit::create_rate_limiter();

    let app = Router::new()
        .route("/", get(|| async { "API E-Pasien Khanza Ready!" }))
        // Endpoint Login & Password
        .route("/api/login", post(handlers::auth::login_pasien))
        .route(
            "/api/pasien/change-password",
            post(handlers::auth::change_password),
        )
        // Endpoint Jadwal Dokter
        .route("/api/jadwal", get(handlers::jadwal::get_jadwal_dokter))
        // Endpoint Booking Online
        .route("/api/booking", post(handlers::booking::create_booking))
        .route(
            "/api/booking/history",
            get(handlers::booking::get_booking_history),
        )
        .route("/api/booking/checkin", post(handlers::booking::check_in))
        .route(
            "/api/booking/cancel",
            post(handlers::booking::cancel_booking),
        )
        // Endpoint Berita RSUD
        .route("/api/news", get(handlers::news::get_news))
        // Endpoint Rekam Medis (SOAP)
        .route(
            "/api/rekam-medis/history",
            get(handlers::rekam_medis::get_rekam_medis_history),
        )
        // Endpoint Surat Kontrol
        .route(
            "/api/surat-kontrol",
            get(handlers::surat_kontrol::get_surat_kontrol_list),
        )
        .route(
            "/api/surat-kontrol/:no_surat",
            get(handlers::surat_kontrol::get_surat_kontrol_detail),
        )
        .route(
            "/api/surat-kontrol/:no_surat/pdf",
            get(handlers::surat_kontrol::download_surat_kontrol_pdf),
        )
        // Endpoint Lab Results
        .route(
            "/api/lab-results",
            get(handlers::lab_results::get_lab_results_list),
        )
        .route(
            "/api/lab-results/:no_rawat",
            get(handlers::lab_results::get_lab_result_detail),
        )
        // Endpoint Radiology Results
        .route(
            "/api/radiology-results",
            get(handlers::radiologi::get_radiology_results_list),
        )
        .route(
            "/api/radiology-results/:no_rawat",
            get(handlers::radiologi::get_radiology_result_detail),
        )
        // Endpoint Cek Kamar
        .route("/api/kamar", get(handlers::kamar::get_ketersediaan_kamar))
        // Endpoint Status Antrian
        .route("/api/antrian", get(handlers::antrian::get_antrian_status))
        // Endpoint Riwayat Obat
        .route("/api/riwayat-obat", get(handlers::riwayat_obat::get_riwayat_obat))
        // Endpoint FCM Token
        .route("/api/fcm/token", post(handlers::notification::register_token))
        // Add FCM client to extensions
        .layer(axum::Extension(fcm_client))
        // Apply rate limiting middleware
        .layer(axum::middleware::from_fn(
            middleware::rate_limit::rate_limit_middleware,
        ))
        // Extension(rate_limiter) harus di luar middleware agar tersedia saat middleware jalan
        .layer(axum::Extension(rate_limiter))
        .layer(cors)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Server e-Pasien jalan di http://localhost:3000");
    println!("⚡ Rate limiting: 100 requests/minute per IP");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
