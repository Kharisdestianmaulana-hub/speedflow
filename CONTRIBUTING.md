# Contributing to SpeedFlow

First off, thank you for taking the time to contribute! Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are greatly appreciated.

Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before contributing to ensure a welcoming and inclusive environment for everyone.

---

## Table of Contents
- [How to Contribute](#how-to-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Enhancements](#suggesting-enhancements)
  - [Pull Requests](#pull-requests)
- [Development Setup](#development-setup)
  - [Prerequisites](#prerequisites)
  - [Building from Source](#building-from-source)
  - [Running Tests](#running-tests)
- [Code Style and Guidelines](#code-style-and-guidelines)
  - [Commit Messages](#commit-messages)
  - [Slint UI Conventions](#slint-ui-conventions)
- [License](#license)

---

## How to Contribute

### Reporting Bugs
Before creating bug reports, please check existing issues to ensure the bug has not already been reported. When creating a bug report, please provide as much relevant information as possible:
- A clear and descriptive title.
- Steps to reproduce the issue.
- Expected behavior versus actual behavior.
- Operating system version (e.g. Windows 11 23H2).
- Application version (e.g. v0.1.0-beta).
- Log output or error messages if applicable.

Please use the [Bug Report Template](.github/ISSUE_TEMPLATE/bug_report.yml).

### Suggesting Enhancements
Feature requests and enhancement suggestions are welcome. Please include:
- A clear description of the proposed feature.
- Why this feature would be useful to SpeedFlow users.
- Possible implementation details or mockups if relevant.

Please use the [Feature Request Template](.github/ISSUE_TEMPLATE/feature_request.yml).

### Pull Requests
1. Fork the repository and create a new branch from `main`.
2. Make your changes with clear, focused commits.
3. Ensure the project builds with zero warnings (`cargo check`).
4. Ensure all unit and integration tests pass (`cargo test`).
5. Open a pull request against the `main` branch with a clear description of your changes.

---

## Development Setup

### Prerequisites
- **Rust:** Rust 1.85.0 or later (2024 edition). Install via [rustup.rs](https://rustup.rs).
- **Git:** Latest version of Git.
- **Inno Setup 6:** Required only if building the Windows installer executable (`SpeedFlow-Setup-v*.exe`).

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/Kharisdestianmaulana-hub/speedflow.git
   cd speedflow
   ```

2. Run the application in development mode:
   ```bash
   cargo run
   ```

3. Build optimized release binary:
   ```bash
   cargo build --release
   ```
   The compiled executable will be located at `target/release/speedflow.exe`.

4. Build complete distribution package (Installer and Portable):
   ```powershell
   .\scripts\package-release.ps1
   ```
   Outputs will be placed in the `dist/` directory.

### Running Tests

Execute the automated test suite:
```bash
cargo test
```

To run with full test output:
```bash
cargo test -- --nocapture
```

---

## Code Style and Guidelines

### Rust Guidelines
- Follow standard Rust idiomatic practices (`clippy` and `rustfmt`).
- Avoid adding heavy dependencies unless strictly necessary.
- Ensure all asynchronous tasks are gracefully cancelable and handle network dropouts robustly.
- Maintain documentation comments (`///`) on public functions and structs.

### Slint UI Conventions
- All UI components reside in `src/ui/`.
- Use the centralized design tokens defined in `src/ui/theme/colors.slint` and `src/ui/theme/tokens.slint`.
- Support both Light and Dark themes for every visual element.
- Keep UI labels in clear, human-friendly English without overly technical jargon.

### Commit Messages
We follow the Conventional Commits specification:
- `feat:` A new feature.
- `fix:` A bug fix.
- `docs:` Documentation only changes.
- `perf:` A code change that improves performance.
- `refactor:` A code change that neither fixes a bug nor adds a feature.
- `test:` Adding missing tests or correcting existing tests.
- `chore:` Changes to the build process or auxiliary tools.

Example:
```text
feat: add bandwidth throttling slider to settings view
fix: handle unexpected connection drops during multi-threaded chunk download
docs: update installation instructions in README
```

---

## License
By contributing to SpeedFlow, you agree that your contributions will be licensed under the project's [GNU General Public License v3.0](LICENSE).
