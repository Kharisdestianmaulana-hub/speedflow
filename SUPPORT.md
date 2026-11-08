# Support and Community Guide

Thank you for using SpeedFlow! We are committed to providing a reliable, fast, and open-source download acceleration experience.

If you have questions, encounter an issue, or need guidance, please consult the resources below.

---

## Getting Help

### 1. GitHub Discussions (Recommended for General Questions)
For questions, usage tips, setup advice, or general conversation about SpeedFlow, please visit our GitHub Discussions:
https://github.com/Kharisdestianmaulana-hub/speedflow/discussions

Recommended discussion categories:
- **Q&A:** Ask questions about settings, thread optimization, or browser configuration.
- **Ideas:** Share thoughts or discuss proposed features before opening a formal request.
- **Show and Tell:** Share your experience, download speeds, and setups with the community.

### 2. GitHub Issues (Bug Reports and Feature Requests)
If you found a bug or want to submit a formal enhancement request, please open a ticket on GitHub Issues using our structured templates:
https://github.com/Kharisdestianmaulana-hub/speedflow/issues

- **Bug Reports:** Use the [Bug Report Form](.github/ISSUE_TEMPLATE/bug_report.yml) to provide details, environment info, and reproduction steps.
- **Feature Requests:** Use the [Feature Request Form](.github/ISSUE_TEMPLATE/feature_request.yml) to outline new capabilities.

### 3. Security Vulnerabilities
If you discover a security vulnerability, please do not disclose it publicly. Review our [Security Policy](SECURITY.md) and report it privately via GitHub Security Advisories:
https://github.com/Kharisdestianmaulana-hub/speedflow/security/advisories/new

---

## Frequently Asked Questions (FAQ)

### How does SpeedFlow accelerate downloads?
SpeedFlow splits files into multiple byte-range segments and downloads them simultaneously using parallel HTTP connections (up to 32 parallel threads). When all segments are downloaded, SpeedFlow merges them into the finished file. This technique maximizes your available internet bandwidth, especially for servers with per-connection bandwidth limits.

### Why does the browser extension connect to 127.0.0.1:19760?
SpeedFlow runs a local HTTP bridge server strictly bound to localhost (`127.0.0.1:19760`). When you initiate a download in your web browser, the extension intercepts the download URL and sends it locally to SpeedFlow. No data is ever sent to external cloud servers; all communication remains entirely private on your own computer.

### How do I install the browser extension?
- **Microsoft Edge and Mozilla Firefox:** Open SpeedFlow Settings and click "Connect Edge & Firefox". SpeedFlow automatically connects the extension via the Windows Registry. Restart your browser to activate.
- **Google Chrome and Brave:** Open `chrome://extensions`, toggle ON "Developer mode" in the top-right corner, click "Load unpacked", and select the `extension` folder inside the SpeedFlow directory (or click "Open Extension Folder" in SpeedFlow Settings).

### Is SpeedFlow free and open source?
Yes. SpeedFlow is free and open-source software licensed under the [GNU General Public License v3.0 (GPL-3.0)](LICENSE). There are no ads, no trackers, and no telemetry.

### Where are downloaded files stored?
By default, completed downloads are saved to your Windows `Downloads` folder. If "Organize Files Automatically by Type" is enabled, SpeedFlow automatically sorts finished files into subfolders: `Video`, `Music`, `Documents`, `Programs`, `Images`, and `Compressed`. You can customize this folder anytime in SpeedFlow Settings.

### What is the difference between Installer and Portable versions?
- **Installer (Setup.exe):** Standard Windows installation wizard that sets up desktop shortcuts, Start Menu shortcuts, and integrates with Windows Installed Apps for easy uninstallation.
- **Portable (.zip):** Self-contained package that requires no installation. Extract the archive and run `speedflow.exe` directly from any folder or USB drive.
