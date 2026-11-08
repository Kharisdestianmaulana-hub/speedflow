# Changelog

All notable changes to SpeedFlow will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned Features (Roadmap)
- **Video and Audio Media Sniffer:** Browser overlay button to capture streaming media with resolution selection (1080p, 720p, MP4, MP3 audio).
- **Automated Download Scheduler:** Configurable start and stop timers for queued downloads, with optional automatic PC Shutdown or Sleep upon queue completion.
- **Flexible Speed Limiter and Profiles:** Per-task transfer rate throttling alongside quick-toggle speed profiles:
  - *Gaming and Browsing Mode:* Caps download speed to 20% to keep network latency low and web browsing responsive.
  - *Full Speed Mode:* Uncapped 100% download acceleration.
- **Official Browser Store Availability:** Direct 1-click installation on Microsoft Edge Add-ons and Mozilla Firefox AMO (submissions currently under store review).

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
