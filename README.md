<p align="center">
  <img src="assets/logo/logo-128.png" alt="SpeedFlow Logo" width="100" height="100" />
</p>

<h1 align="center">SpeedFlow</h1>

<p align="center">
  <strong>High-Performance Desktop Download Manager & Network Accelerator</strong>
  <br />
  Built with Rust and Slint UI
</p>

<p align="center">
  <a href="https://github.com/Kharisdestianmaulana-hub/speedflow/releases"><img src="https://img.shields.io/badge/version-0.1.0--beta-blue.svg?style=flat-square" alt="Version 0.1.0-beta" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-green.svg?style=flat-square" alt="License: GPL-3.0" /></a>
  <img src="https://img.shields.io/badge/Rust-2024%20Edition-DEA584.svg?style=flat-square&logo=rust&logoColor=white" alt="Rust 2024" />
  <img src="https://img.shields.io/badge/GUI-Slint%201.18.1-23C7A2.svg?style=flat-square" alt="Slint 1.18.1" />
  <img src="https://img.shields.io/badge/Platform-Windows-0078D6.svg?style=flat-square&logo=windows&logoColor=white" alt="Platform: Windows" />
  <a href="CONTRIBUTING.md"><img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square" alt="PRs Welcome" /></a>
  <a href="CHANGELOG.md"><img src="https://img.shields.io/badge/changelog-Keep%20a%20Changelog-orange.svg?style=flat-square" alt="Changelog" /></a>
</p>

<br />

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
├── ARCHITECTURE.md             # Subsystem architecture and technical documentation
├── CHANGELOG.md                # Version release history and upcoming roadmap
├── CONTRIBUTING.md             # Contribution guidelines and development setup
├── CODE_OF_CONDUCT.md          # Community standards and code of conduct
├── SECURITY.md                 # Vulnerability disclosure and security policy
├── SUPPORT.md                  # User support resources and FAQ
├── .github/                    # Issue forms and pull request templates
├── assets/                     # Application icons and branding assets
├── extension/                  # WebExtension (Chrome, Edge, Firefox, Brave)
│   ├── manifest.json
│   ├── background.js
│   ├── popup.html
│   └── icons/
├── installer/                  # Inno Setup Windows installer configuration
├── scripts/                    # Automated release packaging PowerShell scripts
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

## Downloads and Installation

SpeedFlow is currently optimized for Windows 10 and Windows 11 (64-bit). You can download the latest official pre-compiled binaries from [GitHub Releases](https://github.com/Kharisdestianmaulana-hub/speedflow/releases):

- **Windows Installer (`SpeedFlow-Setup-v*.exe`):** Recommended for most users. Includes a standard setup wizard with desktop shortcut, Start Menu shortcut, and automatic uninstaller.
- **Portable Version (`SpeedFlow-v*-Portable.zip`):** Standalone archive. Extract the zip file and run `speedflow.exe` directly without installation.

---

## Getting Started

### Prerequisites
- Rust stable toolchain (version 1.80 or higher)
- Cargo package manager
- C++ build tools (required for native platform linking)
- Inno Setup 6 (optional, required only if compiling the Windows installer)

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

4. Generate complete release packages (Installer and Portable):
```powershell
.\scripts\package-release.ps1
```
The output packages will be placed in the `dist/` folder.

### Running Tests
Execute the automated test suite:
```bash
cargo test
```

---

## Browser Extension Installation

1. Open SpeedFlow Settings and click "Open Extension Folder" (or navigate to `extension/` in this repository).
2. In Chromium-based browsers (Chrome, Edge, Brave):
   - Open `chrome://extensions` or `edge://extensions`.
   - Enable "Developer mode" in the upper-right corner.
   - Click "Load unpacked" and select the `extension/` folder.
3. In Firefox:
   - Open `about:debugging#/runtime/this-firefox`.
   - Click "Load Temporary Add-on" and select `extension/manifest.json`.

---

## Roadmap (Upcoming v0.2.0)

The following features are scheduled for the upcoming **v0.2.0** release:

- **Automated Download Scheduler:** Configurable start and stop timers for queued downloads, with optional automatic PC Shutdown or Sleep upon queue completion.
- **Flexible Speed Limiter & Profiles:** Independent per-task transfer rate limits alongside 1-click quick speed profiles:
  - *Gaming and Browsing Mode:* Caps download speed to 20% to keep ping low and browsing responsive.
  - *Full Speed Mode:* Restores 100% uncapped parallel acceleration.
- **Video & Audio Media Sniffer (Preview):** Floating browser overlay widget to detect streaming media with quality selection (1080p, 720p, 480p, MP4, and MP3 audio extraction).
- **Official Browser Store Availability:** Direct 1-click installation on Microsoft Edge Add-ons Store and Mozilla Firefox AMO (currently under store review).

For complete release history and future exploration, see [CHANGELOG.md](CHANGELOG.md).

---

## Community and Documentation

For detailed information about architecture, contributing, release history, and support, please refer to the following guides:

| Document | Description |
|---|---|
| [ARCHITECTURE.md](ARCHITECTURE.md) | In-depth technical architecture, data flow diagrams, and subsystem breakdown |
| [CHANGELOG.md](CHANGELOG.md) | Release history and roadmap for upcoming features |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Development setup, contribution guidelines, and commit standards |
| [SUPPORT.md](SUPPORT.md) | Frequently asked questions, troubleshooting, and community links |
| [SECURITY.md](SECURITY.md) | Vulnerability disclosure policy and reporting process |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Community standards and pledge (Contributor Covenant v2.1) |

---

## License

This project is licensed under the terms of the GNU General Public License v3.0 (GPLv3). See the [LICENSE](LICENSE) file for full details.
