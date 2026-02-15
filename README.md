# Mini HTTP Server

Proyek ini adalah implementasi sederhana dari HTTP server menggunakan bahasa pemrograman Rust. Dibuat sebagai bagian dari proses pembelajaran Rust, proyek ini bertujuan untuk memahami dasar-dasar networking, handling HTTP requests, dan struktur kode Rust.

## Deskripsi

Mini HTTP Server adalah server HTTP minimal yang dapat menangani permintaan dasar. Saat ini, server ini berjalan di `http://127.0.0.1:7878` dan dapat merespons permintaan GET untuk path root (`/`) dan favicon (`/favicon.ico`).

Proyek ini akan terus dikembangkan oleh saya sendiri sebagai sarana pembelajaran dan eksplorasi fitur-fitur Rust yang lebih lanjut.

## Fitur Saat Ini

- Menjalankan server HTTP di localhost pada port 7878
- Handling permintaan GET untuk path `/` dan `/favicon.ico`
- Parsing dasar HTTP request
- Logging sederhana untuk request yang masuk

## Fitur yang Direncanakan untuk Dikembangkan

- **Routing Lanjutan**: Mendukung routing untuk berbagai path dan metode HTTP (POST, PUT, DELETE)
- **Middleware**: Implementasi middleware untuk logging, authentication, dan error handling
- **Static File Serving**: Kemampuan untuk menyajikan file statis seperti HTML, CSS, dan JavaScript
- **Template Engine**: Integrasi dengan template engine untuk rendering halaman dinamis
- **Database Integration**: Koneksi ke database untuk penyimpanan data
- **WebSocket Support**: Dukungan untuk komunikasi real-time menggunakan WebSocket
- **Security Features**: Implementasi HTTPS, CORS, dan validasi input
- **Performance Optimization**: Optimasi untuk handling concurrent connections dan load balancing
- **Testing**: Unit tests dan integration tests yang komprehensif
- **Documentation**: Dokumentasi API dan panduan penggunaan yang lengkap

## Persyaratan Sistem

- Rust (versi terbaru direkomendasikan)
- Cargo (package manager untuk Rust)

## Instalasi dan Setup

1. Clone repository ini:
   ```
   git clone https://github.com/username/mini_http.git
   cd mini_http
   ```

2. Build proyek:
   ```
   cargo build
   ```

## Cara Menjalankan

1. Jalankan server:
   ```
   cargo run
   ```

2. Buka browser dan akses `http://127.0.0.1:7878`

Server akan mulai berjalan dan menampilkan log untuk setiap request yang masuk.

## Struktur Proyek

- `src/main.rs`: Entry point aplikasi
- `src/server.rs`: Implementasi logika server HTTP
- `Cargo.toml`: File konfigurasi dependensi Rust

## Kontribusi

Proyek ini dikembangkan secara pribadi sebagai bagian dari pembelajaran. Namun, saran dan feedback sangat diterima melalui issues di GitHub.

## Lisensi

Proyek ini menggunakan lisensi MIT. Lihat file `LICENSE` untuk detail lebih lanjut.