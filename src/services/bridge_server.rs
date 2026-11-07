use std::net::SocketAddr;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptedDownload {
    pub url: String,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub referrer: Option<String>,
    #[serde(default)]
    pub cookies: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
}

pub struct BridgeServer {
    port: u16,
    event_tx: mpsc::Sender<InterceptedDownload>,
}

impl BridgeServer {
    pub fn new(port: u16, event_tx: mpsc::Sender<InterceptedDownload>) -> Self {
        Self { port, event_tx }
    }

    pub async fn run(self: Arc<Self>) -> anyhow::Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = match TcpListener::bind(addr).await {
            Ok(l) => {
                info!("SpeedFlow Browser Bridge Server listening on http://{}", addr);
                l
            }
            Err(e) => {
                warn!("Could not bind Browser Bridge Server on port {}: {}. Extension integration will be disabled.", self.port, e);
                return Err(e.into());
            }
        };

        loop {
            match listener.accept().await {
                Ok((socket, _)) => {
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(socket).await {
                            // Client disconnected or dropped, trace debug
                            tracing::debug!("Bridge connection closed: {:?}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting bridge connection: {:?}", e);
                }
            }
        }
    }

    async fn handle_connection(&self, mut socket: TcpStream) -> anyhow::Result<()> {
        let mut buffer = vec![0u8; 65536];
        let mut total_read = 0usize;

        loop {
            let n = socket.read(&mut buffer[total_read..]).await?;
            if n == 0 {
                break;
            }
            total_read += n;

            let partial = String::from_utf8_lossy(&buffer[..total_read]);
            if let Some(header_end) = partial.find("\r\n\r\n") {
                let headers_part = &partial[..header_end];
                let body_start = header_end + 4;
                let content_len = headers_part
                    .lines()
                    .find_map(|line| {
                        let lower = line.to_ascii_lowercase();
                        if lower.starts_with("content-length:") {
                            line[15..].trim().parse::<usize>().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);

                if total_read >= body_start + content_len || total_read >= buffer.len() {
                    break;
                }
            } else if total_read >= buffer.len() {
                break;
            }
        }

        if total_read == 0 {
            return Ok(());
        }

        let request = String::from_utf8_lossy(&buffer[..total_read]);
        let mut lines = request.lines();
        let request_line = match lines.next() {
            Some(line) => line,
            None => return Ok(()),
        };

        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("");

        // Handle CORS Preflight
        if method == "OPTIONS" {
            let response = "HTTP/1.1 200 OK\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
Access-Control-Allow-Headers: Content-Type\r\n\
Content-Length: 0\r\n\
Connection: close\r\n\r\n";
            socket.write_all(response.as_bytes()).await?;
            socket.flush().await?;
            return Ok(());
        }

        // Handle Ping / Status Check
        if method == "GET" && (path == "/api/ping" || path == "/ping" || path == "/") {
            let body = r#"{"status":"online","app":"SpeedFlow","version":"0.1.0"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\n\
Access-Control-Allow-Origin: *\r\n\
Content-Type: application/json\r\n\
Content-Length: {}\r\n\
Connection: close\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await?;
            socket.flush().await?;
            return Ok(());
        }

        // Handle Download Interception
        if method == "POST" && (path == "/api/download" || path == "/download") {
            // Find body after \r\n\r\n
            if let Some(pos) = request.find("\r\n\r\n") {
                let body = &request[pos + 4..];
                if let Ok(intercepted) = serde_json::from_str::<InterceptedDownload>(body) {
                    info!("Received intercepted download from browser: {:?}", intercepted.url);
                    let _ = self.event_tx.send(intercepted).await;

                    let res_body = r#"{"status":"ok","message":"Intercepted by SpeedFlow"}"#;
                    let response = format!(
                        "HTTP/1.1 200 OK\r\n\
Access-Control-Allow-Origin: *\r\n\
Content-Type: application/json\r\n\
Content-Length: {}\r\n\
Connection: close\r\n\r\n{}",
                        res_body.len(),
                        res_body
                    );
                    socket.write_all(response.as_bytes()).await?;
                    socket.flush().await?;
                    return Ok(());
                }
            }

            let err_body = r#"{"status":"error","message":"Invalid payload"}"#;
            let response = format!(
                "HTTP/1.1 400 Bad Request\r\n\
Access-Control-Allow-Origin: *\r\n\
Content-Type: application/json\r\n\
Content-Length: {}\r\n\
Connection: close\r\n\r\n{}",
                err_body.len(),
                err_body
            );
            socket.write_all(response.as_bytes()).await?;
            socket.flush().await?;
            return Ok(());
        }

        // 404 Not Found
        let not_found = "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
        socket.write_all(not_found.as_bytes()).await?;
        socket.flush().await?;
        Ok(())
    }
}
