use reqwest::header::{
    ACCEPT, ACCEPT_LANGUAGE, ACCEPT_RANGES, CONNECTION, CONTENT_DISPOSITION, CONTENT_LENGTH,
    CONTENT_RANGE, COOKIE, ETAG, REFERER, USER_AGENT,
};
use reqwest::Client;
use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;
use std::time::Duration;
use url::Url;

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Default)]
pub struct SessionContext {
    pub referrer: Option<String>,
    pub cookies: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RemoteFileInfo {
    pub url: String,
    pub filename: String,
    pub content_length: Option<u64>,
    pub supports_range: bool,
    pub etag: Option<String>,
}

pub struct HttpEngine {
    client: Client,
    sessions: RwLock<HashMap<String, SessionContext>>,
    info_cache: RwLock<HashMap<String, RemoteFileInfo>>,
}

impl HttpEngine {
    pub fn new() -> Self {
        // High-speed multi-socket configuration:
        // Force HTTP/1.1 (`http1_only`) so every parallel range segment opens a separate
        // physical TCP connection instead of being multiplexed onto a single HTTP/2 stream.
        let client = Client::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .http1_only()
            .tcp_nodelay(true)
            .pool_max_idle_per_host(32)
            .pool_idle_timeout(Duration::from_secs(90))
            .connect_timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            sessions: RwLock::new(HashMap::new()),
            info_cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Store browser session headers (Referer, Cookies, User-Agent) captured by the extension
    pub fn register_session(&self, raw_url: &str, ctx: SessionContext) {
        if let Ok(mut map) = self.sessions.write() {
            map.insert(raw_url.to_string(), ctx.clone());
            if let Ok(parsed) = Url::parse(raw_url) {
                if let Some(host) = parsed.host_str() {
                    map.insert(host.to_lowercase(), ctx);
                }
            }
        }
    }

    pub fn get_session(&self, raw_url: &str) -> Option<SessionContext> {
        if let Ok(map) = self.sessions.read() {
            if let Some(ctx) = map.get(raw_url) {
                return Some(ctx.clone());
            }
            if let Ok(parsed) = Url::parse(raw_url) {
                if let Some(host) = parsed.host_str() {
                    if let Some(ctx) = map.get(&host.to_lowercase()) {
                        return Some(ctx.clone());
                    }
                }
            }
        }
        None
    }

    /// Build a GET request pre-populated with browser session headers (Cookie, Referer, User-Agent)
    pub fn build_get(&self, raw_url: &str) -> reqwest::RequestBuilder {
        let mut req = self
            .client
            .get(raw_url)
            .header(ACCEPT, "*/*")
            .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9,id;q=0.8")
            .header(CONNECTION, "keep-alive");

        let session = self.get_session(raw_url);
        let mut has_referer = false;

        if let Some(ctx) = session {
            if let Some(ua) = ctx.user_agent.as_deref().filter(|s| !s.trim().is_empty()) {
                req = req.header(USER_AGENT, ua.trim());
            }
            if let Some(refr) = ctx.referrer.as_deref().filter(|s| !s.trim().is_empty()) {
                req = req.header(REFERER, refr.trim());
                has_referer = true;
            }
            if let Some(ck) = ctx.cookies.as_deref().filter(|s| !s.trim().is_empty()) {
                req = req.header(COOKIE, ck.trim());
            }
        }

        if !has_referer {
            if let Ok(parsed) = Url::parse(raw_url) {
                if let Some(host) = parsed.host_str() {
                    let origin_ref = format!("{}://{}/", parsed.scheme(), host);
                    req = req.header(REFERER, origin_ref);
                }
            }
        }

        req
    }

    fn apply_session_headers(
        &self,
        raw_url: &str,
        mut req: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        req = req
            .header(ACCEPT, "*/*")
            .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9,id;q=0.8")
            .header(CONNECTION, "keep-alive");

        let session = self.get_session(raw_url);
        let mut has_referer = false;

        if let Some(ctx) = session {
            if let Some(ua) = ctx.user_agent.as_deref().filter(|s| !s.trim().is_empty()) {
                req = req.header(USER_AGENT, ua.trim());
            }
            if let Some(refr) = ctx.referrer.as_deref().filter(|s| !s.trim().is_empty()) {
                req = req.header(REFERER, refr.trim());
                has_referer = true;
            }
            if let Some(ck) = ctx.cookies.as_deref().filter(|s| !s.trim().is_empty()) {
                req = req.header(COOKIE, ck.trim());
            }
        }

        if !has_referer {
            if let Ok(parsed) = Url::parse(raw_url) {
                if let Some(host) = parsed.host_str() {
                    let origin_ref = format!("{}://{}/", parsed.scheme(), host);
                    req = req.header(REFERER, origin_ref);
                }
            }
        }

        req
    }

    pub async fn inspect_url(&self, raw_url: &str) -> anyhow::Result<RemoteFileInfo> {
        if let Ok(cache) = self.info_cache.read() {
            if let Some(cached) = cache.get(raw_url) {
                return Ok(cached.clone());
            }
        }

        let parsed_url = Url::parse(raw_url)?;
        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            anyhow::bail!("Only HTTP and HTTPS protocols are supported");
        }

        // Try GET with Range: bytes=0-0 first so single-use redirect tokens
        // are not wasted by HEAD, and we immediately verify true byte-range support + total size.
        let response = match self
            .build_get(raw_url)
            .header("Range", "bytes=0-0")
            .send()
            .await
        {
            Ok(res)
                if res.status().is_success()
                    || res.status() == reqwest::StatusCode::PARTIAL_CONTENT =>
            {
                res
            }
            _ => {
                self.apply_session_headers(raw_url, self.client.head(raw_url))
                    .send()
                    .await?
            }
        };

        let final_url = response.url().to_string();
        if final_url != raw_url {
            if let Some(ctx) = self.get_session(raw_url) {
                self.register_session(&final_url, ctx);
            }
        }

        let status = response.status();
        let headers = response.headers().clone();

        // If response is 206 Partial Content (1 byte for bytes=0-0), consume the 1-byte body
        // so hyper keeps the underlying TCP socket warm in the connection pool instead of closing it!
        if status == reqwest::StatusCode::PARTIAL_CONTENT {
            let _ = response.bytes().await;
        } else {
            drop(response);
        }

        // 1. Check Range support
        let mut supports_range = status == reqwest::StatusCode::PARTIAL_CONTENT;
        if let Some(val) = headers.get(ACCEPT_RANGES) {
            if let Ok(val_str) = val.to_str() {
                if val_str.to_lowercase().contains("bytes") {
                    supports_range = true;
                }
            }
        }

        // 2. Content Length (prefer Content-Range total `/TOTAL` when response is 206 Partial Content)
        let mut content_length = headers
            .get(CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.rsplit('/').next())
            .and_then(|tot| tot.trim().parse::<u64>().ok());

        if content_length.is_none() {
            let cl = headers
                .get(CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());
            // Only use Content-Length if not a 1-byte range probe
            if status != reqwest::StatusCode::PARTIAL_CONTENT || cl.unwrap_or(0) > 1 {
                content_length = cl;
            }
        }

        // 3. ETag
        let etag = headers
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // 4. Filename extraction (Header -> Final URL Query -> Original URL Query -> URL Path)
        let mut filename = String::new();
        if let Some(disp) = headers.get(CONTENT_DISPOSITION) {
            if let Ok(disp_str) = disp.to_str() {
                if let Some(idx) = disp_str.find("filename=") {
                    let raw_fn = &disp_str[idx + 9..];
                    let trimmed = raw_fn
                        .split(';')
                        .next()
                        .unwrap_or(raw_fn)
                        .trim_matches(|c| c == '"' || c == '\'' || c == ' ');
                    if !trimmed.is_empty() {
                        filename = trimmed.to_string();
                    }
                }
            }
        }

        let final_parsed = Url::parse(&final_url).unwrap_or_else(|_| parsed_url.clone());
        for check_url in [&final_parsed, &parsed_url] {
            if !filename.is_empty() {
                break;
            }
            for (k, v) in check_url.query_pairs() {
                let key_lower = k.to_lowercase();
                if (key_lower == "filename" || key_lower == "file" || key_lower == "name")
                    && !v.trim().is_empty()
                {
                    filename = v.trim().to_string();
                    break;
                } else if key_lower == "response-content-disposition" {
                    if let Some(idx) = v.find("filename=") {
                        let raw_fn = &v[idx + 9..];
                        let trimmed = raw_fn
                            .split(';')
                            .next()
                            .unwrap_or(raw_fn)
                            .trim_matches(|c| c == '"' || c == '\'' || c == ' ');
                        if !trimmed.is_empty() {
                            filename = trimmed.to_string();
                            break;
                        }
                    }
                }
            }
        }

        if filename.is_empty() {
            if let Some(segments) = final_parsed.path_segments() {
                if let Some(last) = segments.last() {
                    if !last.is_empty() {
                        filename = last.to_string();
                    }
                }
            }
        }

        if filename.is_empty() {
            filename = "downloaded_file.bin".to_string();
        }

        // Sanitize filename
        let sanitized_filename = Path::new(&filename)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("downloaded_file.bin")
            .to_string();

        let info = RemoteFileInfo {
            url: final_url.clone(),
            filename: sanitized_filename,
            content_length,
            supports_range,
            etag,
        };

        if let Ok(mut cache) = self.info_cache.write() {
            cache.insert(raw_url.to_string(), info.clone());
            cache.insert(final_url, info.clone());
        }

        Ok(info)
    }
}
