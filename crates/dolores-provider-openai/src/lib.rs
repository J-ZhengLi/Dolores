use async_trait::async_trait;
use dolores_core::{ConnectionPreferences, Message, ModelProvider, PluginDescriptor};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use url::{Host, Url};

const MAX_FRAME_BYTES: usize = 1024 * 1024;

pub struct OpenAiProvider {
    client: reqwest::Client,
    endpoint: Url,
    model: String,
    api_key: String,
}

pub fn validate_preferences(preferences: &ConnectionPreferences) -> Result<Url, String> {
    if preferences.model.trim().is_empty() || preferences.model.len() > 200 {
        return Err("Enter a model ID (up to 200 bytes).".into());
    }
    if preferences.base_url.len() > 2048 {
        return Err("Endpoint URL is too long.".into());
    }
    let mut url = Url::parse(preferences.base_url.trim())
        .map_err(|_| "Enter a valid endpoint URL.".to_string())?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(
            "Keep credentials, query parameters and fragments out of the endpoint URL.".into(),
        );
    }
    let local = match url.host() {
        Some(Host::Domain(host)) => host == "localhost",
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        None => false,
    };
    if url.scheme() != "https" && !(url.scheme() == "http" && local) {
        return Err("Use HTTPS, or HTTP with a loopback address for a local model.".into());
    }
    let path = format!("{}/chat/completions", url.path().trim_end_matches('/'));
    url.set_path(&path);
    Ok(url)
}

impl OpenAiProvider {
    pub fn new(preferences: &ConnectionPreferences, api_key: String) -> Result<Self, String> {
        let endpoint = validate_preferences(preferences)?;
        if api_key.len() > 4096 || api_key.contains(['\r', '\n']) {
            return Err("Invalid API key.".into());
        }
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(180))
            .build()
            .map_err(|_| "Could not initialize the connection.".to_string())?;
        Ok(Self {
            client,
            endpoint,
            model: preferences.model.trim().into(),
            api_key,
        })
    }
}

#[derive(Default)]
struct SseDecoder {
    buffer: Vec<u8>,
}
impl SseDecoder {
    fn push(&mut self, bytes: &[u8]) -> Result<Vec<String>, String> {
        self.buffer.extend_from_slice(bytes);
        let mut frames = Vec::new();
        loop {
            let delimiter = self
                .buffer
                .windows(2)
                .position(|w| w == b"\n\n")
                .map(|i| (i, 2));
            let crlf = self
                .buffer
                .windows(4)
                .position(|w| w == b"\r\n\r\n")
                .map(|i| (i, 4));
            let boundary = match (delimiter, crlf) {
                (Some(a), Some(b)) => Some(if a.0 < b.0 { a } else { b }),
                (a, b) => a.or(b),
            };
            let Some((index, length)) = boundary else {
                break;
            };
            if index > MAX_FRAME_BYTES {
                return Err("Provider stream frame exceeds the limit.".into());
            }
            let frame = String::from_utf8(self.buffer.drain(..index + length).collect())
                .map_err(|_| "Provider returned invalid UTF-8.".to_string())?;
            let data = frame
                .lines()
                .filter_map(|line| {
                    line.strip_prefix("data:")
                        .map(|text| text.strip_prefix(' ').unwrap_or(text))
                })
                .collect::<Vec<_>>()
                .join("\n");
            if !data.is_empty() {
                frames.push(data);
            }
        }
        if self.buffer.len() > MAX_FRAME_BYTES {
            return Err("Provider stream frame exceeds the limit.".into());
        }
        Ok(frames)
    }
}

#[async_trait]
impl ModelProvider for OpenAiProvider {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "dolores.provider.openai-compatible",
            kind: "provider",
            api_version: 1,
        }
    }
    async fn stream(
        &self,
        messages: Vec<Message>,
        output: mpsc::Sender<String>,
        cancel: CancellationToken,
    ) -> Result<(), String> {
        let mut request = self.client.post(self.endpoint.clone()).json(&json!({ "model": self.model, "messages": messages, "stream": true, "max_tokens": 2048 }));
        if !self.api_key.is_empty() {
            request = request.bearer_auth(&self.api_key);
        }
        let response = tokio::select! {
            _ = cancel.cancelled() => return Err("Response stopped.".into()),
            result = request.send() => result.map_err(|_| "Could not reach the model. Check the endpoint and whether the server is running.".to_string())?,
        };
        if !response.status().is_success() {
            return Err(match response.status().as_u16() {
                401 | 403 => "Model access denied. Check your API key and permissions.".into(),
                404 => "Model endpoint not found. Check the base URL and model ID.".into(),
                429 => "Model rate limit reached. Try again later.".into(),
                status => {
                    format!("Model request failed (HTTP {status}). Check your connection settings.")
                }
            });
        }
        if !response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.starts_with("text/event-stream"))
        {
            return Err("Provider did not return a text/event-stream response.".into());
        }
        let mut stream = response.bytes_stream();
        let mut decoder = SseDecoder::default();
        let mut finished = false;
        loop {
            let chunk = tokio::select! {
                _ = cancel.cancelled() => return Err("Response stopped.".into()),
                next = stream.next() => next,
            };
            let Some(chunk) = chunk else { break };
            for data in decoder.push(&chunk.map_err(|_| {
                "Connection interrupted before the response finished.".to_string()
            })?)? {
                if data == "[DONE]" {
                    return Ok(());
                }
                let value: Value = serde_json::from_str(&data)
                    .map_err(|_| "Provider returned a malformed stream.".to_string())?;
                if value.get("error").is_some() {
                    return Err("Provider reported an error while streaming.".into());
                }
                let Some(choice) = value
                    .get("choices")
                    .and_then(Value::as_array)
                    .and_then(|choices| choices.first())
                else {
                    continue;
                };
                if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
                    if reason == "length" {
                        return Err(
                            "Response reached the model output limit. Try a shorter request."
                                .into(),
                        );
                    }
                    if reason != "stop" {
                        return Err("Provider finished without a complete text response.".into());
                    }
                    finished = true;
                }
                if let Some(content) = choice.pointer("/delta/content").and_then(Value::as_str) {
                    if !content.is_empty() {
                        tokio::select! {
                            _ = cancel.cancelled() => return Err("Response stopped.".into()),
                            result = output.send(content.into()) => result.map_err(|_| "Conversation window closed.".to_string())?,
                        }
                    }
                }
            }
        }
        if finished && decoder.buffer.is_empty() {
            Ok(())
        } else {
            Err("Connection ended before the response finished.".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    #[test]
    fn decodes_fragmented_unicode_and_crlf() {
        let wire =
            "data: {\"choices\":[{\"delta\":{\"content\":\"你好\"}}]}\r\n\r\ndata: [DONE]\n\n";
        let mut decoder = SseDecoder::default();
        let mut frames = Vec::new();
        for byte in wire.as_bytes() {
            frames.extend(decoder.push(&[*byte]).unwrap());
        }
        assert_eq!(frames.len(), 2);
        assert!(frames[0].contains("你好"));
        assert_eq!(frames[1], "[DONE]");
    }
    #[test]
    fn rejects_secret_urls_and_remote_plaintext() {
        for base_url in [
            "http://example.com/v1",
            "https://user:secret@example.com/v1",
            "https://example.com/v1?key=secret",
            "file:///v1",
        ] {
            assert!(validate_preferences(&ConnectionPreferences {
                base_url: base_url.into(),
                model: "test".into()
            })
            .is_err());
        }
        assert!(validate_preferences(&ConnectionPreferences {
            base_url: "http://[::1]:8080/v1/".into(),
            model: "test".into()
        })
        .is_ok());
    }
    #[test]
    fn rejects_oversized_stream_frames() {
        assert!(SseDecoder::default()
            .push(&vec![b'x'; MAX_FRAME_BYTES + 1])
            .is_err());
    }
    async fn server(response: &'static str) -> (String, tokio::task::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut bytes = Vec::new();
            loop {
                let mut chunk = [0u8; 4096];
                let count = socket.read(&mut chunk).await.unwrap();
                if count == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..count]);
                if let Some(index) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..index]);
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            line.to_ascii_lowercase()
                                .strip_prefix("content-length: ")
                                .map(|n| n.parse().unwrap())
                        })
                        .unwrap_or(0);
                    if bytes.len() >= index + 4 + length {
                        break;
                    }
                }
            }
            socket.write_all(response.as_bytes()).await.unwrap();
            String::from_utf8(bytes).unwrap()
        });
        (format!("http://{address}/v1"), task)
    }
    #[tokio::test]
    async fn streams_from_real_http_and_sends_expected_contract() {
        let (base_url, server) = server("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\ndata: [DONE]\n\n").await;
        let provider = OpenAiProvider::new(
            &ConnectionPreferences {
                base_url,
                model: "local-test".into(),
            },
            "test-key".into(),
        )
        .unwrap();
        let (tx, mut rx) = mpsc::channel(32);
        provider
            .stream(
                vec![Message {
                    role: dolores_core::Role::User,
                    content: "hi".into(),
                }],
                tx,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(rx.recv().await.as_deref(), Some("hello"));
        let request = server.await.unwrap();
        assert!(request.starts_with("POST /v1/chat/completions "));
        assert!(request
            .to_ascii_lowercase()
            .contains("authorization: bearer test-key"));
        let body: Value = serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["stream"], true);
        assert_eq!(body["messages"][0]["content"], "hi");
    }
    #[tokio::test]
    async fn rejects_truncated_and_business_error_streams() {
        for body in [
            "data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n",
            "data: {\"error\":{\"message\":\"secret\"}}\n\n",
        ] {
            let response: &'static str = Box::leak(format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n{body}").into_boxed_str());
            let (base_url, task) = server(response).await;
            let provider = OpenAiProvider::new(
                &ConnectionPreferences {
                    base_url,
                    model: "test".into(),
                },
                String::new(),
            )
            .unwrap();
            let (tx, _receiver) = mpsc::channel(32);
            let error = provider
                .stream(vec![], tx, CancellationToken::new())
                .await
                .unwrap_err();
            assert!(!error.contains("secret"));
            task.await.unwrap();
        }
    }
    #[tokio::test]
    async fn cancelled_request_does_not_wait_for_network() {
        let provider = OpenAiProvider::new(
            &ConnectionPreferences {
                base_url: "http://127.0.0.1:9/v1".into(),
                model: "test".into(),
            },
            String::new(),
        )
        .unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let (tx, _) = mpsc::channel(32);
        let result = tokio::time::timeout(
            Duration::from_millis(100),
            provider.stream(vec![], tx, cancel),
        )
        .await
        .unwrap();
        assert!(result.is_err());
    }
}
