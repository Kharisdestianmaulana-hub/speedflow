# SpeedFlow ⚡

> **Desktop Download Manager & Accelerator**  
> Built with Rust, Slint UI, and Tokio.

SpeedFlow adalah aplikasi desktop modern yang dirancang untuk mengoptimalkan proses pengunduhan file dari internet melalui koneksi paralel (*HTTP Range Requests*), pemulihan unduhan (*pause/resume*), manajemen antrean cerdas, dan pemantauan kecepatan *real-time*.

---

## 🚀 Fitur Utama

- **Akselerasi Download:** Membagi file menjadi beberapa segmen independen (hingga 16 koneksi) untuk memaksimalkan throughput jaringan.
- **Dukungan Pause & Resume:** Melanjutkan proses unduhan yang terputus dengan validasi ETag dan integritas segmen.
- **Queue Management:** Mengatur antrean unduhan berdasarkan prioritas dengan batas unduhan simultan yang dapat dikonfigurasi.
- **Bandwidth Limiter:** Mengatur batas kecepatan agregat agar tidak membebani jaringan pengguna.
- **Smart Utilities:**
  - Clipboard URL Monitor: Deteksi otomatis URL unduhan yang disalin ke clipboard.
  - Smart File Categorization: Pengelompokan file otomatis berdasarkan tipe (Documents, Compressed, Video, Audio, Programs, Images).
  - Checksum Verification: Validasi integritas file menggunakan SHA-256.
  - Download Scheduler: Penjadwalan waktu mulai unduhan otomatis.
- **Antarmuka Modern (Slint):**
  - Desain minimalis dan fungsional berbasis Slint.
  - Aksen visual **Green Accent** (`#78C375`).
  - Dukungan tema **Light**, **Dark**, dan **System**.

---

## 🛠️ Tech Stack

| Komponen | Teknologi | Deskripsi |
|---|---|---|
| **Bahasa Utama** | Rust | Core application, safety, and high-performance networking |
| **GUI Framework** | Slint | Declarative desktop GUI toolkit |
| **Runtime Asinkron** | Tokio | Multi-threaded async runtime |
| **HTTP Client** | Reqwest | HTTP/HTTPS range requests & streaming |
| **Penyimpanan Lokal** | SQLite + SQLx | Database lokal untuk riwayat dan metadata |
| **Serialisasi** | Serde / Serde JSON | Format data terstruktur |
| **Diagnostik** | Tracing | Structured logging |

---

## 📂 Struktur Project

```text
speedflow/
├── Cargo.toml
├── build.rs
├── README.md
├── SRS.md
├── PRD.md
├── Design_System.md
├── migrations/
│   └── 001_initial_schema.sql
├── src/
│   ├── main.rs
│   ├── app/        # State, event, dan command koordinasi
│   ├── core/       # Task model, queue, scheduler, events
│   ├── engine/     # HTTP range engine, segment manager, writer, retry, limiter
│   ├── storage/    # SQLite database, models, repository
│   ├── services/   # Clipboard, file classifier, checksum, notifications
│   ├── platform/   # Abstraksi OS (file explorer, system opener)
│   └── ui/         # Slint UI (theme, components, layouts, pages)
└── tests/          # Integration & unit test suite
```

---

## 💻 Cara Menjalankan

### Persyaratan
- Rust 1.80+ (dengan Cargo)
- Compiler C++ / CMake (jika dibutuhkan oleh backend native Slint)

### Menjalankan Aplikasi
```bash
cargo run
```

### Menjalankan Pengujian
```bash
cargo test
```

---

## 📄 Lisensi
Independent Software Project. Semua hak cipta dilindungi.
