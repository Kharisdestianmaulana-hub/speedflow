# SpeedFlow

High-Performance Desktop Download Manager and Network Accelerator built with Rust and Slint UI.

SpeedFlow is an open-source desktop application engineered for high-throughput, reliable, and resource-efficient file downloads. By leveraging asynchronous multi-connection chunking, in-flight work-stealing segment splits, and a lightweight native user interface, SpeedFlow provides substantial network acceleration without the memory footprint of web-based desktop wrappers.

---

## Core Capabilities

### 1. Network Acceleration and Dynamic Segmentation
- Parallel Chunk Downloading: Divides files into multiple independent byte-range requests (up to 32 concurrent connections per file) to saturate network bandwidth.
- In-Flight Work Stealing: Automatically detects lagging or remaining large file chunks and splits them dynamically across idle worker threads.
- Robust Resume and Recovery: Validates HTTP ETag and Accept-Ranges headers to seamlessly resume interrupted downloads without data corruption.
- Real-Time Bandwidth Limiting: Integrated token-bucket speed limiter to regulate total aggregate bandwidth consumption.
- Update Download Link: Allows renewing expired download URLs for paused or failed tasks while preserving existing segment progress.

### 2. Browser Integration and IPC Bridge
- Cross-Browser Extension Support: Native Manifest V3 extensions for Google Chrome, Microsoft Edge, Mozilla Firefox, and Brave.
- Localhost IPC Bridge: Secure local bridge listening on port 19760 to intercept browser downloads with complete session headers (Cookies, Referer, User-Agent).
- Standalone File Info Dialog: Opens a focused popup window upon browser link interception without forcing the main application window into view.

### 3. Desktop Native Architecture
- Zero-Bloat GUI: Built using the Slint declarative UI toolkit, compiling directly to native machine code with zero Electron/Chromium overhead.
- Native Windows System Tray: Minimizes to the notification area with a dedicated Win32 tray loop and context menu.
- Background Notifications: Dispatches native Windows notifications when downloads begin or complete while running in tray mode, routing directly to the target tab upon user interaction.
- Integrity Checksums: Background SHA-256 calculation upon download completion with single-click clipboard copying.

---

## Technology Stack

| Layer | Component | Description |
|---|---|---|
| Core Language | Rust (2024 Edition) | Memory-safe, high-concurrency runtime with zero garbage collection |
| UI Framework | Slint (v1.18.1) | Declarative, hardware-accelerated desktop user interface |
| Async Runtime | Tokio | Multi-threaded asynchronous I/O and task scheduling |
| Networking | Reqwest & Hyper | Optimized HTTP/1.1 socket pooling and streaming |
| Persistence | SQLite via SQLx | ACID-compliant local database for queue, settings, and task history |
| Logging | Tracing | Structured, context-aware diagnostic logging |

---

## Project Structure

```text
SpeedFlow/
├── Cargo.toml                  # Cargo manifest and dependency definitions
├── build.rs                    # Slint UI ahead-of-time compiler build script
├── README.md                   # Project documentation
├── assets/                     # Application icons and branding assets
├── extension/                  # WebExtension (Chrome, Edge, Firefox, Brave)
│   ├── manifest.json
│   ├── background.js
│   └── popup.html
├── migrations/                 # SQLite schema migration scripts
├── src/
│   ├── main.rs                 # Application entrypoint, event loop, and bridge routing
│   ├── app/                    # Application service coordination and state management
│   ├── core/                   # Task lifecycles, queue ordering, and scheduling
│   ├── engine/                 # Parallel HTTP range engine, segment writer, speed limiter
│   ├── platform/               # Native OS integrations (Win32 tray, explorer, autostart)
│   ├── services/               # Browser bridge server, clipboard, file classifier, notifications
│   ├── storage/                # SQLite database connections, models, and repositories
│   └── ui/                     # Slint markup files (views, components, themes)
└── tests/                      # Integration and regression test suites
```

---

## Getting Started

### Prerequisites
- Rust stable toolchain (version 1.80 or higher)
- Cargo package manager
- C++ build tools (required for native platform linking)

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/Kharisdestianmaulana-hub/speedflow.git
cd speedflow
```

2. Compile in debug mode:
```bash
cargo build --bin speedflow
```

3. Compile optimized release binary:
```bash
cargo build --release --bin speedflow
```
The compiled executable will be located at `target/release/speedflow.exe`.

### Running Tests
Execute the comprehensive automated test suite:
```bash
cargo test
```

---

## Browser Extension Installation

1. Navigate to the `extension/` directory in this repository.
2. In Chromium-based browsers (Chrome, Edge, Brave):
   - Open `chrome://extensions` or `edge://extensions`.
   - Enable "Developer mode" in the upper-right corner.
   - Click "Load unpacked" and select the `extension/` folder.
3. In Firefox:
   - Open `about:debugging#/runtime/this-firefox`.
   - Click "Load Temporary Add-on" and select `extension/manifest.json`.

---

## License

This project is licensed under the terms of the GNU General Public License v3.0 (GPLv3). See the [LICENSE](LICENSE) file for full details.
