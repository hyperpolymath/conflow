// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
//! Gossamer Groove endpoint for conflow.
//!
//! Exposes conflow's config-orchestration capabilities via the groove
//! discovery protocol. Any groove-aware system (Gossamer, PanLL, etc.)
//! can discover conflow by probing GET /.well-known/groove on port 7700.
//!
//! conflow works standalone as a CLI tool. When groove consumers connect,
//! they gain access to conflow's pipeline orchestration, config validation,
//! and RSR compliance checking.
//!
//! The groove connector types are formally verified in Gossamer's Groove.idr:
//! - IsSubset proves consumers can only connect if conflow satisfies their needs
//! - GrooveHandle is linear: consumers MUST disconnect (no dangling grooves)
//!
//! ## Groove Protocol
//!
//! - `GET  /.well-known/groove` — Capability manifest (JSON)
//! - `GET  /health`             — Simple health check
//!
//! ## Capabilities Offered
//!
//! - `config-orchestration` — Pipeline execution for CUE, Nickel, and config workflows
//!
//! ## Capabilities Consumed (enhanced when available)
//!
//! - `octad-storage` (from VeriSimDB) — Persist pipeline results and compliance reports

use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{error, info, warn};

/// Maximum HTTP request size (16 KiB).
const MAX_REQUEST_SIZE: usize = 16 * 1024;

/// Run the groove discovery HTTP server on the given port.
///
/// This is a minimal HTTP server that handles only the groove protocol
/// endpoints. Invoked by `conflow serve --port 7700`.
pub async fn run(port: u16) -> miette::Result<()> {
    let addr: SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .map_err(|e| miette::miette!("Failed to parse bind address: {}", e))?;

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| miette::miette!("Failed to bind to {}: {}", addr, e))?;
    info!("Groove endpoint listening on {}", addr);
    info!("Probe: curl http://localhost:{}/.well-known/groove", port);

    loop {
        match listener.accept().await {
            Ok((mut stream, _peer)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_request(&mut stream).await {
                        warn!("Groove request error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Groove accept error: {}", e);
            }
        }
    }
}

/// Build the groove manifest JSON for conflow.
fn manifest(port: u16) -> String {
    format!(
        r#"{{
  "groove_version": "1",
  "service_id": "conflow",
  "service_version": "{}",
  "capabilities": {{
    "config_orchestration": {{
      "type": "config-orchestration",
      "description": "Pipeline orchestration for CUE, Nickel, and configuration validation workflows",
      "protocol": "http",
      "endpoint": "/api/v1/config",
      "requires_auth": false,
      "panel_compatible": true
    }}
  }},
  "consumes": ["octad-storage"],
  "endpoints": {{
    "api": "http://localhost:{}/api/v1",
    "health": "http://localhost:{}/health"
  }},
  "health": "/health",
  "applicability": ["individual", "team"]
}}"#,
        env!("CARGO_PKG_VERSION"),
        port,
        port
    )
}

/// Handle a single groove HTTP request.
async fn handle_request(
    stream: &mut tokio::net::TcpStream,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = vec![0u8; MAX_REQUEST_SIZE];
    let n = stream.read(&mut buf).await?;
    let request = std::str::from_utf8(&buf[..n])?;

    let first_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        send_response(stream, 400, "text/plain", "Bad Request").await?;
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    match (method, path) {
        // GET /.well-known/groove — Return the capability manifest.
        ("GET", "/.well-known/groove") => {
            let json = manifest(7700);
            send_response(stream, 200, "application/json", &json).await?;
        }

        // GET /health — Simple health check.
        ("GET", "/health") => {
            send_response(
                stream,
                200,
                "application/json",
                r#"{"status":"ok","service":"conflow"}"#,
            )
            .await?;
        }

        // Unknown route.
        _ => {
            send_response(stream, 404, "text/plain", "Not Found").await?;
        }
    }

    Ok(())
}

/// Send an HTTP response with the given content type and body.
async fn send_response(
    stream: &mut tokio::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Unknown",
    };
    let response = format!(
        "HTTP/1.0 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        status_text,
        content_type,
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}
