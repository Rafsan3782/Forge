use std::io::{self, Read, Write};

/// Content-Length header prefix matching standard LSP framing
const CONTENT_LENGTH_HEADER: &str = "Content-Length: ";

/// Encode a UTF-8 JSON message with Content-Length framing:
/// `Content-Length: <len>\r\n\r\n<payload>`
pub fn encode_lsp_message(payload: &str) -> Vec<u8> {
    let len = payload.len();
    let header = format!("{}{}\r\n\r\n", CONTENT_LENGTH_HEADER, len);
    let mut out = Vec::with_capacity(header.len() + len);
    out.extend_from_slice(header.as_bytes());
    out.extend_from_slice(payload.as_bytes());
    out
}

/// Helper to write an LSP framed message to any std::io::Write
pub fn write_lsp_message<W: Write>(writer: &mut W, payload: &str) -> io::Result<()> {
    let framed = encode_lsp_message(payload);
    writer.write_all(&framed)?;
    writer.flush()
}

/// Buffer parser for stream-oriented reading of Content-Length framed JSON-RPC messages.
#[derive(Default, Debug)]
pub struct LspFramingBuffer {
    buffer: Vec<u8>,
}

impl LspFramingBuffer {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Push newly read bytes into internal buffer
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// Attempt to parse and extract the next complete payload from the buffer.
    /// Returns `Ok(Some(String))` when a message is completed,
    /// `Ok(None)` when more data is needed,
    /// or `Err(String)` if framing headers are malformed.
    pub fn pop_message(&mut self) -> Result<Option<String>, String> {
        loop {
            // Find end of header (\r\n\r\n or \n\n)
            let header_end_pos = match self.find_header_end() {
                Some(pos) => pos,
                None => return Ok(None),
            };

            let header_bytes = &self.buffer[..header_end_pos.header_len];
            let header_str = std::str::from_utf8(header_bytes)
                .map_err(|e| format!("Invalid UTF-8 in LSP headers: {e}"))?;

            // Parse Content-Length from headers
            let mut content_length: Option<usize> = None;
            for line in header_str.lines() {
                let trimmed = line.trim();
                if let Some(val_str) = trimmed.strip_prefix(CONTENT_LENGTH_HEADER) {
                    let len = val_str
                        .trim()
                        .parse::<usize>()
                        .map_err(|e| format!("Invalid Content-Length header '{val_str}': {e}"))?;
                    content_length = Some(len);
                    break;
                }
            }

            let length = match content_length {
                Some(l) => l,
                None => {
                    // Header without Content-Length: discard up to separator and error
                    self.buffer.drain(..header_end_pos.total_header_bytes);
                    return Err("Missing Content-Length header in LSP framing".to_string());
                }
            };

            // Check if full payload has arrived
            let total_needed = header_end_pos.total_header_bytes + length;
            if self.buffer.len() < total_needed {
                return Ok(None);
            }

            // Extract payload
            let payload_start = header_end_pos.total_header_bytes;
            let payload_end = payload_start + length;
            let payload_bytes = &self.buffer[payload_start..payload_end];
            let payload_str = std::str::from_utf8(payload_bytes)
                .map_err(|e| format!("Invalid UTF-8 in LSP payload: {e}"))?
                .to_string();

            // Drain processed message from buffer
            self.buffer.drain(..payload_end);
            return Ok(Some(payload_str));
        }
    }

    fn find_header_end(&self) -> Option<HeaderEndPosition> {
        // Look for \r\n\r\n
        for i in 0..self.buffer.len().saturating_sub(3) {
            if &self.buffer[i..i + 4] == b"\r\n\r\n" {
                return Some(HeaderEndPosition {
                    header_len: i,
                    total_header_bytes: i + 4,
                });
            }
        }
        // Fallback for \n\n
        for i in 0..self.buffer.len().saturating_sub(1) {
            if &self.buffer[i..i + 2] == b"\n\n" {
                return Some(HeaderEndPosition {
                    header_len: i,
                    total_header_bytes: i + 2,
                });
            }
        }
        None
    }
}

struct HeaderEndPosition {
    header_len: usize,
    total_header_bytes: usize,
}

/// Reads a single complete LSP framed message synchronously from a reader.
pub fn read_lsp_message<R: Read>(reader: &mut R) -> io::Result<Option<String>> {
    let mut header_buf = Vec::new();
    let mut byte = [0u8; 1];

    // Read headers byte-by-byte until \r\n\r\n or EOF
    loop {
        let n = reader.read(&mut byte)?;
        if n == 0 {
            if header_buf.is_empty() {
                return Ok(None); // Normal EOF
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "EOF while reading LSP headers",
                ));
            }
        }
        header_buf.push(byte[0]);
        let len = header_buf.len();
        if len >= 4 && &header_buf[len - 4..] == b"\r\n\r\n" {
            break;
        }
    }

    let header_str = std::str::from_utf8(&header_buf[..header_buf.len() - 4])
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut content_length: Option<usize> = None;
    for line in header_str.lines() {
        let trimmed = line.trim();
        if let Some(val_str) = trimmed.strip_prefix(CONTENT_LENGTH_HEADER) {
            let len = val_str
                .trim()
                .parse::<usize>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            content_length = Some(len);
            break;
        }
    }

    let length = content_length.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Missing Content-Length in LSP header",
        )
    })?;

    let mut body = vec![0u8; length];
    reader.read_exact(&mut body)?;

    let body_str = String::from_utf8(body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(Some(body_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_lsp_message() {
        let payload = r#"{"jsonrpc":"2.0","method":"ping"}"#;
        let encoded = encode_lsp_message(payload);
        let expected = format!("Content-Length: {}\r\n\r\n{}", payload.len(), payload);
        assert_eq!(String::from_utf8(encoded).unwrap(), expected);
    }

    #[test]
    fn test_buffer_pop_single_message() {
        let mut buffer = LspFramingBuffer::new();
        let payload = r#"{"type":"test","data":123}"#;
        let encoded = encode_lsp_message(payload);

        buffer.push_bytes(&encoded);
        let msg = buffer.pop_message().unwrap();
        assert_eq!(msg, Some(payload.to_string()));

        // Buffer should now be empty
        let next = buffer.pop_message().unwrap();
        assert_eq!(next, None);
    }

    #[test]
    fn test_buffer_pop_partial_chunks() {
        let mut buffer = LspFramingBuffer::new();
        let payload = r#"{"type":"partial"}"#;
        let encoded = encode_lsp_message(payload);

        // Push partial header
        buffer.push_bytes(&encoded[..5]);
        assert_eq!(buffer.pop_message().unwrap(), None);

        // Push rest of header and half body
        let mid = encoded.len() - 3;
        buffer.push_bytes(&encoded[5..mid]);
        assert_eq!(buffer.pop_message().unwrap(), None);

        // Push remaining 3 bytes
        buffer.push_bytes(&encoded[mid..]);
        assert_eq!(buffer.pop_message().unwrap(), Some(payload.to_string()));
    }

    #[test]
    fn test_buffer_pop_multiple_concatenated_messages() {
        let mut buffer = LspFramingBuffer::new();
        let payload1 = r#"{"id":1}"#;
        let payload2 = r#"{"id":2}"#;

        let mut data = encode_lsp_message(payload1);
        data.extend_from_slice(&encode_lsp_message(payload2));

        buffer.push_bytes(&data);
        assert_eq!(buffer.pop_message().unwrap(), Some(payload1.to_string()));
        assert_eq!(buffer.pop_message().unwrap(), Some(payload2.to_string()));
        assert_eq!(buffer.pop_message().unwrap(), None);
    }

    #[test]
    fn test_read_lsp_message_stream() {
        let payload = r#"{"run_id":"test_123"}"#;
        let encoded = encode_lsp_message(payload);
        let mut cursor = std::io::Cursor::new(encoded);

        let msg = read_lsp_message(&mut cursor).unwrap();
        assert_eq!(msg, Some(payload.to_string()));

        // Next read gives None (EOF)
        let eof = read_lsp_message(&mut cursor).unwrap();
        assert_eq!(eof, None);
    }
}
