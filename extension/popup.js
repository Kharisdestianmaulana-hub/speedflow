const PING_URL = "http://127.0.0.1:19760/api/ping";

document.addEventListener("DOMContentLoaded", async () => {
  const statusBadge = document.getElementById("statusBadge");
  const statusDot = document.getElementById("statusDot");
  const statusText = document.getElementById("statusText");
  const interceptToggle = document.getElementById("interceptToggle");

  // 1. Load stored toggle state
  chrome.storage.local.get(["enabled"], (res) => {
    if (res.enabled !== undefined) {
      interceptToggle.checked = res.enabled;
    }
  });

  interceptToggle.addEventListener("change", () => {
    chrome.storage.local.set({ enabled: interceptToggle.checked });
  });

  // 2. Ping SpeedFlow Desktop Bridge Server
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 1500);

    const response = await fetch(PING_URL, { signal: controller.signal });
    clearTimeout(timeoutId);

    if (response.ok) {
      statusBadge.className = "status-badge status-online";
      statusDot.className = "dot dot-online";
      statusText.textContent = "Engine Online";
    } else {
      throw new Error("Bridge responded with non-200");
    }
  } catch (err) {
    statusBadge.className = "status-badge status-offline";
    statusDot.className = "dot dot-offline";
    statusText.textContent = "App Offline";
  }
});
