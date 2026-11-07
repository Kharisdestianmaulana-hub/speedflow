// SpeedFlow Browser Download Interceptor - Background Service Worker
const SPEEDFLOW_BRIDGE_URL = "http://127.0.0.1:19760/api/download";
const SPEEDFLOW_PING_URL = "http://127.0.0.1:19760/api/ping";

// Default intercepted extensions
const DEFAULT_EXTENSIONS = [
  "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso", "dmg",
  "exe", "msi", "apk", "deb", "rpm", "bin",
  "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm",
  "mp3", "wav", "flac", "aac", "ogg", "m4a",
  "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx"
];

// Initialize default settings on install
chrome.runtime.onInstalled.addListener(() => {
  chrome.storage.local.get(["enabled", "extensions", "minSizeMb"], (res) => {
    if (res.enabled === undefined) {
      chrome.storage.local.set({ enabled: true });
    }
    if (!res.extensions) {
      chrome.storage.local.set({ extensions: DEFAULT_EXTENSIONS });
    }
    if (res.minSizeMb === undefined) {
      chrome.storage.local.set({ minSizeMb: 0 }); // 0 means intercept all sizes
    }
  });
  console.log("[SpeedFlow] Download Interceptor installed and ready.");
});

// Download creation listener
chrome.downloads.onCreated.addListener(async (downloadItem) => {
  try {
    const config = await chrome.storage.local.get(["enabled", "extensions", "minSizeMb"]);
    if (config.enabled === false) {
      return; // Interception disabled by user
    }

    const url = downloadItem.finalUrl || downloadItem.url;
    if (!url) return;

    // Ignore browser internal schemes
    if (url.startsWith("blob:") || url.startsWith("data:") || url.startsWith("chrome:") || url.startsWith("edge:") || url.startsWith("about:")) {
      return;
    }

    // Check file extension (prefer explicit query param ?filename= before URL path)
    let filename = downloadItem.filename ? downloadItem.filename.split(/[\\/]/).pop() : "";
    if (!filename) {
      try {
        const parsed = new URL(url);
        const queryFn = parsed.searchParams.get("filename") ||
                        parsed.searchParams.get("file") ||
                        parsed.searchParams.get("name");
        if (queryFn && queryFn.trim().length > 0) {
          filename = queryFn.trim();
        } else {
          filename = parsed.pathname.split("/").pop() || "";
        }
      } catch (e) {
        // invalid url
      }
    }

    const extMatch = filename.match(/\.([0-9a-z]+)(?:[\?#]|$)/i);
    const ext = extMatch ? extMatch[1].toLowerCase() : "";

    const allowedExtensions = config.extensions || DEFAULT_EXTENSIONS;
    const isExtensionMatched = allowedExtensions.includes(ext) || ext === "";

    // If extension is not in list and has an extension, don't intercept (unless unknown)
    if (ext && !isExtensionMatched) {
      return;
    }

    // Check size limit if available
    const minSize = (config.minSizeMb || 0) * 1024 * 1024;
    if (minSize > 0 && downloadItem.fileSize > 0 && downloadItem.fileSize < minSize) {
      return;
    }

    console.log(`[SpeedFlow] Intercepting download: ${filename || url}`);

    // Cancel browser download immediately
    chrome.downloads.cancel(downloadItem.id, () => {
      chrome.downloads.erase({ id: downloadItem.id });
    });

    // Gather browser session cookies for the download URL & referrer (IDM-style session forwarding)
    let cookieString = "";
    try {
      if (chrome.cookies && chrome.cookies.getAll) {
        const cookieMap = new Map();
        if (downloadItem.referrer) {
          try {
            const refCookies = await chrome.cookies.getAll({ url: downloadItem.referrer });
            for (const c of refCookies) {
              cookieMap.set(c.name, c.value);
            }
          } catch (_) {}
        }
        try {
          const urlCookies = await chrome.cookies.getAll({ url: url });
          for (const c of urlCookies) {
            cookieMap.set(c.name, c.value);
          }
        } catch (_) {}
        if (cookieMap.size > 0) {
          cookieString = Array.from(cookieMap.entries())
            .map(([k, v]) => `${k}=${v}`)
            .join("; ");
        }
      }
    } catch (e) {
      console.warn("[SpeedFlow] Failed to read cookies:", e);
    }

    // Send to SpeedFlow Desktop Bridge
    const payload = {
      url: url,
      filename: filename || null,
      referrer: downloadItem.referrer || null,
      cookies: cookieString || null,
      user_agent: navigator.userAgent || null
    };

    const response = await fetch(SPEEDFLOW_BRIDGE_URL, {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify(payload)
    });

    if (response.ok) {
      console.log(`[SpeedFlow] Successfully dispatched download to SpeedFlow app.`);
    } else {
      console.warn(`[SpeedFlow] Bridge returned error HTTP ${response.status}`);
    }
  } catch (err) {
    console.warn(`[SpeedFlow] Could not reach SpeedFlow Desktop app (is it running?):`, err);
  }
});
