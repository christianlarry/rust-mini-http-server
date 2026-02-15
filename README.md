# Mini HTTP Framework

🦀 **Sebuah HTTP framework dari nol (from scratch) menggunakan Rust murni tanpa dependencies eksternal seperti hyper atau axum.**

Proyek ini adalah implementasi mini HTTP framework yang dibangun untuk tujuan pembelajaran mendalam tentang:
- Fundamental HTTP dan networking di Rust
- Ownership, borrowing, dan lifetime dalam konteks server
- Design pattern untuk web framework
- Low-level TCP connection handling

Terinspirasi dari Express.js/NestJS, tetapi dibangun secara low-level dengan Rust `std::net`.

---

## 📑 Table of Contents

- [Tujuan Proyek](#-tujuan-proyek)
- [Fitur Saat Ini](#-fitur-saat-ini)
- [Development Roadmap](#️-development-roadmap)
- [Quick Start](#-quick-start)
- [Struktur Proyek](#-struktur-proyek)
- [Konsep Arsitektur](#-konsep-arsitektur)
- [Learning Goals](#-learning-goals)
- [Current Status](#-current-status)
- [Resources & Inspirasi](#-resources--inspirasi)

---

## 🎯 Tujuan Proyek

- Memahami cara kerja HTTP server dari bawah ke atas
- Mendesain arsitektur framework yang clean dan modular
- Eksplorasi konsep concurrency dan async programming
- Membangun sesuatu yang usable dan reusable

## ✨ Fitur Saat Ini

- ✅ TCP server berbasis `std::net::TcpListener`
- ✅ HTTP request parsing dasar (method dan path)
- ✅ Struct `Request` untuk representasi request
- ✅ Struct `Response` dengan method `send()`
- ✅ Router berbasis `HashMap` dengan format `"METHOD:/path"`
- ✅ Handler function untuk setiap route
- ✅ App abstraction sebagai facade
- ✅ Modular file structure (request, response, router, server, app)
- ✅ Support basic GET routing
- ✅ 404 fallback untuk route tidak ditemukan
- ✅ Clean modular architecture

## 🗺️ Development Roadmap

### Phase 1: Core Stability

Memperkuat fondasi framework sebelum menambah fitur baru.

- [ ] **Complete HTTP Request Parsing** - Parse headers, query params, dan HTTP version
- [ ] **Request Body Parsing** - Support untuk plain text, JSON, dan form-urlencoded
- [ ] **Response Headers** - Tambah method untuk set custom headers (Content-Type, Cache-Control, dll)
- [ ] **Multiple Response Types** - Support `.json()`, `.html()`, `.status()`, dan `.send_file()`
- [ ] **Error Handling System** - Proper error types dan Result-based handling
- [ ] **Unit Tests** - Test coverage untuk parser, router, dan response builder
- [ ] **Code Documentation** - Rustdoc comments untuk public API

### Phase 2: HTTP Improvement

Dukungan penuh untuk HTTP methods dan fitur standard.

- [ ] **POST Method Support** - Termasuk body parsing dan Content-Length handling
- [ ] **PUT & DELETE Methods** - Lengkapi CRUD operations
- [ ] **Request Method Extractor** - Helper untuk extract query params, headers, body dengan mudah
- [ ] **Cookie Support** - Parse dan set cookies
- [ ] **Content Negotiation** - Handle Accept headers dan response sesuai client
- [ ] **Path Parameters** - Dynamic routing seperti `/users/:id`
- [ ] **Query String Parser** - Parse URL query strings ke HashMap atau struct

### Phase 3: Architecture Upgrade

Tingkatkan arsitektur untuk scalability dan extensibility.

- [ ] **Middleware System** - Chain of responsibility untuk pre/post request handling
- [ ] **Middleware: Logger** - Request logging dengan timestamp, method, path, status
- [ ] **Middleware: CORS** - Cross-Origin Resource Sharing support
- [ ] **Context/State Sharing** - Share data antar middleware dan handlers
- [ ] **Route Groups** - Prefix routing untuk modularitas (e.g., `/api/v1`)
- [ ] **Closure-based Handlers** - Support `Box<dyn Fn(Request) -> Response>`
- [ ] **Error Middleware** - Centralized error handling dan custom error pages

### Phase 4: Performance & Concurrency

Optimasi untuk production-ready performance.

- [ ] **Thread Pool** - Handle concurrent connections dengan worker threads
- [ ] **Connection Keep-Alive** - Reuse TCP connections untuk multiple requests
- [ ] **Request Timeout** - Prevent hanging connections
- [ ] **Graceful Shutdown** - Clean server stop tanpa drop connections
- [ ] **Benchmarking Setup** - Measure throughput dan latency
- [ ] **Memory Profiling** - Identify dan fix memory leaks
- [ ] **Async Refactor** - Migrate ke Tokio untuk async I/O (opsional, learning async Rust)

### Phase 5: Advanced Features

Fitur tambahan untuk production usability.

- [ ] **Static File Serving** - Serve HTML, CSS, JS, images dari folder public
- [ ] **Template Engine Integration** - Optional: Tera atau Handlebars
- [ ] **Multipart Form Support** - File upload handling
- [ ] **WebSocket Support** - Upgrade HTTP ke WebSocket connection
- [ ] **HTTPS/TLS Support** - Secure connections dengan rustls
- [ ] **Compression** - Gzip/Brotli response compression
- [ ] **Rate Limiting** - Protect dari spam requests
- [ ] **Session Management** - In-memory atau redis-backed sessions

### Phase 6: Future Vision

Evolusi jangka panjang untuk framework maturity.

- [ ] **Refactor ke Reusable Crate** - Publish ke crates.io
- [ ] **CLI Tool** - Generator untuk boilerplate project (`mini-http new my-app`)
- [ ] **Plugin System** - Extensible architecture untuk third-party plugins
- [ ] **ORM Integration** - Database layer abstraction
- [ ] **Production Examples** - Real-world example apps (REST API, CRUD, auth)
- [ ] **Comprehensive Documentation** - Website dengan guides, tutorials, API docs
- [ ] **Community & Ecosystem** - Accept contributions dan build ecosystem

## 🚀 Quick Start

### Instalasi

```bash
git clone https://github.com/username/mini_http.git
cd mini_http
```

### Setup Environment

Buat file `.env` di root project:

```env
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
```

### Jalankan Server

```bash
cargo run
```

Server akan berjalan di `http://127.0.0.1:8080`

### Basic Usage

```rust
use crate::app::App;

fn main() {
    let mut app = App::new();

    // Register route
    app.get("/", |_, res| {
        res.send("Hello, World!");
    });

    app.get("/about", |_, res| {
        res.send("About page");
    });

    // Start server
    app.run("127.0.0.1:8080");
}
```

## 🛠️ Development

### Build

```bash
cargo build
```

### Run dengan watch mode

```bash
cargo watch -x run
```

### Testing (coming soon)

```bash
cargo test
```

## 📚 Persyaratan Sistem

- Rust 1.70+ (edition 2021)
- Cargo
- (Optional) cargo-watch untuk development

## 📁 Struktur Proyek

```
mini_http/
├── src/
│   ├── main.rs      # Entry point & routing setup
│   ├── app.rs       # App facade & high-level API
│   ├── server.rs    # TCP listener & connection handling
│   ├── router.rs    # Route matching & handler execution
│   ├── request.rs   # HTTP request parsing & representation
│   └── response.rs  # HTTP response builder & sender
├── Cargo.toml       # Dependencies & project metadata
└── README.md        # Project documentation
```

## 🧠 Konsep Arsitektur

### Request Flow
```
TCP Connection → Server → Router → Handler → Response → Client
                    ↓         ↓
                 Parse    Match Route
```

### Komponen Utama

- **App**: Facade untuk register routes dan start server
- **Server**: Menangani TCP connections dan lifecycle
- **Router**: HashMap-based routing dengan key `"METHOD:/path"`
- **Request**: Parsed HTTP request dengan method, path, headers, body
- **Response**: Builder pattern untuk construct HTTP response

## 🎓 Learning Goals

Setiap fase development fokus pada aspek Rust yang berbeda:

1. **Ownership & Borrowing**: Mengelola TcpStream dan buffer handling
2. **Trait & Generics**: Abstraksi untuk handlers dan middleware
3. **Error Handling**: Result, Option, dan custom error types
4. **Concurrency**: Thread pool dan channel communication
5. **Async Programming**: Tokio runtime dan async/await (phase 4)
6. **API Design**: Builder pattern, fluent interface, type safety

## 📊 Current Status

**Phase:** Core Stability (Phase 1)  
**Completion:** ~30% of Phase 1  
**Next Milestone:** Complete HTTP parsing dan response types

## 🤝 Kontribusi

Proyek ini adalah personal learning project yang dikembangkan secara aktif. Fokus utama adalah pembelajaran dan eksplorasi Rust.

Namun demikian, saran, feedback, dan diskusi sangat diterima! Feel free untuk:
- 🐛 Report bugs via Issues
- 💡 Suggest features atau improvements
- 📖 Contribute ke dokumentasi
- ⭐ Star repository jika kamu merasa ini bermanfaat

## 📝 Catatan

- Project ini **tidak production-ready** - ini adalah learning project
- Tidak menggunakan external framework dengan sengaja untuk deep learning
- Performance bukan prioritas utama (untuk sekarang)
- Code mungkin tidak optimal - bagian dari proses pembelajaran

## 📖 Resources & Inspirasi

- [The Rust Book - Building a Multithreaded Web Server](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html)
- [HTTP/1.1 Specification (RFC 7230-7235)](https://httpwg.org/specs/)
- Express.js & NestJS design patterns
- Rust web frameworks: Actix-web, Axum, Rocket (untuk referensi API design)

## 📄 Lisensi

MIT License - bebas digunakan untuk pembelajaran dan eksperimen.

---

**Built with 🦀 Rust and ❤️ for learning**