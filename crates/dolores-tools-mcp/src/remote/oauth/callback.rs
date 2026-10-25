//! A single foreground attempt owns this listener. Dropping its future closes it.
use super::{Failure, Kind, Operation, PendingAuthorization, Result, Tokens};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::Instant,
};
use tokio_util::sync::CancellationToken;

pub struct LoopbackCallback {
    listener: TcpListener,
    redirect: String,
    deadline: Instant,
}
impl LoopbackCallback {
    pub async fn bind() -> Result<Self> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .map_err(|_| Failure(Kind::Callback))?;
        let address = listener.local_addr().map_err(|_| Failure(Kind::Callback))?;
        Ok(Self {
            listener,
            redirect: format!("http://{address}/oauth/callback"),
            deadline: Instant::now() + Duration::from_secs(300),
        })
    }
    pub fn redirect_uri(&self) -> &str {
        &self.redirect
    }
    pub async fn complete(
        self,
        pending: PendingAuthorization,
        cancel: CancellationToken,
    ) -> Result<Tokens> {
        if pending.redirect_uri() != self.redirect {
            return Err(Failure(Kind::Callback));
        }
        let wait = async {
            // Unrelated probes cannot keep an attempt alive indefinitely.
            for _ in 0..8 {
                let (mut socket, peer) = self
                    .listener
                    .accept()
                    .await
                    .map_err(|_| Failure(Kind::Callback))?;
                if !peer.ip().is_loopback() {
                    continue;
                }
                let read = tokio::time::timeout(
                    Duration::from_secs(3),
                    request(&mut socket, &self.redirect),
                )
                .await;
                let callback = match read {
                    Ok(Ok(callback)) => callback,
                    _ => {
                        respond(&mut socket, false).await;
                        continue;
                    }
                };
                match pending.code(&callback) {
                    Ok(_) => {
                        respond(&mut socket, true).await;
                        // No listener survives the one-use exchange.
                        drop(self.listener);
                        return pending
                            .complete(
                                &callback,
                                &mut Operation::until(cancel.clone(), self.deadline),
                            )
                            .await;
                    }
                    Err(error @ Failure(Kind::Denied)) => {
                        respond(&mut socket, false).await;
                        return Err(error);
                    }
                    Err(_) => respond(&mut socket, false).await,
                }
            }
            Err(Failure(Kind::Callback))
        };
        tokio::select! {
            biased;
            _ = cancel.cancelled() => Err(Failure(Kind::Cancelled)),
            _ = tokio::time::sleep_until(self.deadline) => Err(Failure(Kind::AttemptExpired)),
            result = wait => result,
        }
    }
}
async fn request(socket: &mut TcpStream, redirect: &str) -> Result<String> {
    let mut bytes = Vec::new();
    let end = loop {
        let mut buffer = [0; 1024];
        let count = socket
            .read(&mut buffer)
            .await
            .map_err(|_| Failure(Kind::Callback))?;
        if count == 0 || bytes.len() + count > 8192 {
            return Err(Failure(Kind::Callback));
        }
        bytes.extend_from_slice(&buffer[..count]);
        if let Some(end) = bytes.windows(4).position(|s| s == b"\r\n\r\n") {
            break end + 4;
        }
    };
    if end != bytes.len() {
        return Err(Failure(Kind::Callback));
    }
    let headers = std::str::from_utf8(&bytes).map_err(|_| Failure(Kind::Callback))?;
    let mut lines = headers.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split(' ');
    let (Some("GET"), Some(path), Some("HTTP/1.1"), None) =
        (first.next(), first.next(), first.next(), first.next())
    else {
        return Err(Failure(Kind::Callback));
    };
    if !path.starts_with("/oauth/callback?") || path.contains('#') {
        return Err(Failure(Kind::Callback));
    }
    let authority = redirect
        .strip_prefix("http://")
        .and_then(|s| s.split('/').next())
        .ok_or(Failure(Kind::Callback))?;
    let mut host = None;
    for line in lines.filter(|s| !s.is_empty()) {
        let (key, value) = line.split_once(':').ok_or(Failure(Kind::Callback))?;
        if key.eq_ignore_ascii_case("host") {
            if host.replace(value.trim()).is_some() {
                return Err(Failure(Kind::Callback));
            }
        } else if key.eq_ignore_ascii_case("transfer-encoding")
            || key.eq_ignore_ascii_case("content-length") && value.trim() != "0"
        {
            return Err(Failure(Kind::Callback));
        }
    }
    if host != Some(authority) {
        return Err(Failure(Kind::Callback));
    }
    Ok(format!("http://{authority}{path}"))
}
async fn respond(socket: &mut TcpStream, accepted: bool) {
    let (status, body) = if accepted {
        (
            "200 OK",
            "Sign-in received. Return to Dolores to review the connection.",
        )
    } else {
        (
            "400 Bad Request",
            "Sign-in was not accepted. Return to Dolores and try Sign in again.",
        )
    };
    let response = format!("HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'none'; frame-ancestors 'none'\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}", body.len());
    let _ = tokio::time::timeout(
        Duration::from_secs(1),
        socket.write_all(response.as_bytes()),
    )
    .await;
}
