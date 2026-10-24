use super::{fail, Failure, Kind, FRAME, WIRE};
use reqwest::Response;
use serde_json::Value;

#[derive(Default)]
pub(super) struct Sse {
    line: Vec<u8>,
    data: Vec<u8>,
    cr: bool,
    first: bool,
}
impl Sse {
    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<Value>, Failure> {
        let mut messages = vec![];
        for &b in bytes {
            if b == b'\n' && self.cr {
                self.cr = false;
                continue;
            }
            self.cr = b == b'\r';
            if b == b'\r' || b == b'\n' {
                let mut line = std::mem::take(&mut self.line);
                if !self.first {
                    self.first = true;
                    if line.starts_with(&[0xef, 0xbb, 0xbf]) {
                        line.drain(..3);
                    }
                }
                if line.is_empty() {
                    if !self.data.is_empty() {
                        self.data.pop();
                        if !self.data.is_empty() {
                            messages.push(parse(&self.data)?);
                        }
                        self.data.clear();
                    }
                } else if !line.starts_with(b":") {
                    let (key, value) = line
                        .iter()
                        .position(|b| *b == b':')
                        .map_or((line.as_slice(), &[][..]), |p| (&line[..p], &line[p + 1..]));
                    if key == b"data" {
                        let value = value.strip_prefix(b" ").unwrap_or(value);
                        self.data.extend_from_slice(value);
                        self.data.push(b'\n');
                    }
                }
            } else {
                self.line.push(b);
            }
            if self.line.len() > FRAME || self.data.len() > FRAME {
                return Err(fail(Kind::FrameLimit));
            }
        }
        Ok(messages)
    }
}
pub(super) fn parse(bytes: &[u8]) -> Result<Value, Failure> {
    if bytes.len() > FRAME {
        return Err(fail(Kind::FrameLimit));
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| fail(Kind::Protocol))?;
    if !value.is_object() || value["jsonrpc"] != "2.0" {
        return Err(fail(Kind::Protocol));
    }
    Ok(value)
}
pub(super) async fn chunk(
    response: &mut Response,
    wire: &mut usize,
) -> Result<Option<Vec<u8>>, Failure> {
    let chunk = response.chunk().await.map_err(|_| fail(Kind::Network))?;
    if let Some(bytes) = &chunk {
        *wire = wire
            .checked_add(bytes.len())
            .ok_or_else(|| fail(Kind::WireLimit))?;
        if *wire > WIRE {
            return Err(fail(Kind::WireLimit));
        }
    }
    Ok(chunk.map(|b| b.to_vec()))
}
