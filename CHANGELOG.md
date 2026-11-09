# Changelog

All notable changes to SpeedFlow will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased] (Target: v0.2.0)

### Planned Features (v0.2.0 Milestone)
- **Automated Download Scheduler (Stable):** Configurable automated start and stop schedules for queued downloads, featuring post-download power actions:
  - Automatic PC Shutdown upon queue completion.
  - Automatic Sleep / Hibernate upon queue completion.
  - Flexible day filters: Every day, workdays (Monday to Friday), or single run.
- **Flexible Speed Limiter & Profiles (Stable):** Independent per-task bandwidth throttling alongside 1-click quick speed profiles:
  - *Gaming and Browsing Mode:* Caps SpeedFlow transfer rate to 20% of maximum capacity, maintaining low latency (ping) and responsive web browsing.
  - *Full Speed Mode:* Restores 100% uncapped parallel acceleration.
- **Video & Audio Media Sniffer (Preview / Experimental):** Floating browser overlay widget to automatically detect streaming video and audio:
  - Resolution selector: 1080p (Full HD), 720p (HD), 480p, and 360p (MP4 container).
  - Audio extraction: Direct streaming audio conversion to MP3 format.
- **Official Browser Store Availability:** 1-click direct installation from official stores:
  - Microsoft Edge Add-ons Store (submission currently under review).
  - Mozilla Firefox Add-ons / AMO (submission currently under review).

### Fixes & Improvements (v0.2.0 Milestone)
- **Embedded Win32 Application Icon:** Added `winres` resource compilation in `build.rs` to embed `speedflow.ico` directly into the PE executable binary. Fixes generic blank document icon in Windows Search, Start Menu, Taskbar, and Alt-Tab.
- **Windows Startup Experience & Notification:**
  - Added a non-intrusive system notification ("SpeedFlow Running in Background") upon Windows startup to clearly inform users that SpeedFlow is active in the tray without stealing screen focus. Clicking the notification opens the main window.
  - Unified autostart handling across the Windows Registry (`HKCU\...\Run`) and the user Startup folder (`%APPDATA%\...\Startup`), ensuring clean toggling in Settings without orphaned shortcuts.
  - Updated System Tray icon loader in `tray.rs` to retrieve the icon directly from the embedded binary resource, eliminating blank tray icons on portable installations.

### Future Roadmap (v0.3.0+)
- **Auto-Update Checker:** Seamless in-app notification when a new release is published on GitHub Releases.
- **Batch / Bulk Downloader:** Multi-URL paste support and sequential URL pattern generator (`file_[01-50].zip`).
- **Floating Mini Desktop Widget:** Lightweight transparent desktop widget supporting drag-and-drop link interception from any active screen.

---

## [0.1.0-beta] - 2026-11-08

### Added
- **Multi-threaded Acceleration Engine:** High-performance segmented byte-range downloads supporting up to 32 parallel connections per file.
- **Flat Monolithic Desktop UI:** Desktop-first high-density interface built with Slint, featuring seamless runtime switching between Dark and Light themes.
- **Universal Browser Extension:** Cross-browser Manifest V3 extension intercepting downloads from Google Chrome, Brave, Microsoft Edge, and Mozilla Firefox.
- **Localhost Bridge Server:** Privacy-focused local HTTP bridge server listening on `127.0.0.1:19760` with zero external cloud dependencies.
- **Windows Registry Integration:** Automatic 1-click extension registration for Microsoft Edge and Mozilla Firefox via Windows Current User registry (`HKCU`).
- **Embedded Extension Fallback:** Automatic extraction of embedded extension assets into local AppData if external folders are absent.
- **Smart Category Sorting:** Automatic file sorting into dedicated subfolders (`Video`, `Music`, `Documents`, `Programs`, `Images`, `Compressed`).
- **SQLite Database Persistence:** Asynchronous non-blocking task queue, state tracking, and download history powered by SQLx and SQLite.
- **Integrity Checksums:** Automatic SHA-256 hash calculation for completed downloads with 1-click copy feedback.
- **First-Run Onboarding Wizard:** Interactive 4-step configuration guide introducing new users to themes, destination folders, thread limits, and browser setup.
- **Windows Platform Integration:** Native Win32 System Tray minimizing, background download execution, and native Windows shell balloon notifications.
- **Dual Release Packaging:** Automated packaging for both Inno Setup Windows Installer (`SpeedFlow-Setup-v*.exe`) and standalone portable archive (`SpeedFlow-v*-Portable.zip`).

[Unreleased]: https://github.com/Kharisdestianmaulana-hub/speedflow/compare/v0.1.0...HEAD
[0.1.0-beta]: https://github.com/Kharisdestianmaulana-hub/speedflow/releases/tag/v0.1.0-beta
