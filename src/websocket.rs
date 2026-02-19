//! WebSocket support.
//!
//! Provides WebSocket upgrade handshake and bidirectional message framing
//! over an established TCP connection. Supports text and binary messages,
//! ping/pong, and close frames.

use std::io::{Read, Write};

use base64::Engine;
use sha1::{Digest, Sha1};

use crate::error::{Error, Result};

/// The WebSocket magic GUID used in the handshake.
const WS_MAGIC: &str = "258EAFA5-E914-47DA-95CA-5AB5DC085B7";

/// WebSocket message types.
#[derive(Debug, Clone)]
pub enum Message {
    /// Text message (UTF-8).
    Text(String),
    /// Binary message.
    Binary(Vec<u8>),
    /// Ping (heartbeat request).
    Ping(Vec<u8>),
    /// Pong (heartbeat response).
    Pong(Vec<u8>),
    /// Connection close.
    Close,
}

/// WebSocket opcodes.
#[repr(u8)]
enum Opcode {
    Continue = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl Opcode {
    fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x0 => Some(Opcode::Continue),
            0x1 => Some(Opcode::Text),
            0x2 => Some(Opcode::Binary),
            0x8 => Some(Opcode::Close),
            0x9 => Some(Opcode::Ping),
            0xA => Some(Opcode::Pong),
            _ => None,
        }
    }
}

/// A WebSocket connection for bidirectional communication.
///
/// Created by the framework when a WebSocket upgrade is accepted.
/// Provides methods to read and write WebSocket messages.
pub struct WebSocket {
    stream: Box<dyn ReadWrite>,
}

/// Combined Read + Write trait for stream abstraction.
pub trait ReadWrite: Read + Write + Send {}
impl<T: Read + Write + Send> ReadWrite for T {}

impl WebSocket {
    /// Create a WebSocket from a raw stream (called internally after handshake).
    pub fn new(stream: Box<dyn ReadWrite>) -> Self {
        WebSocket { stream }
    }

    /// Compute the WebSocket accept key from the client key.
    pub fn compute_accept_key(client_key: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(client_key.trim().as_bytes());
        hasher.update(WS_MAGIC.as_bytes());
        let hash = hasher.finalize();
        base64::engine::general_purpose::STANDARD.encode(hash)
    }

    /// Generate the HTTP response for a WebSocket upgrade handshake.
    pub fn handshake_response(client_key: &str) -> Vec<u8> {
        let accept = Self::compute_accept_key(client_key);
        format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\
             \r\n",
            accept
        )
        .into_bytes()
    }

    /// Read the next WebSocket message.
    ///
    /// Returns `None` if the connection is closed cleanly.
    pub fn read_message(&mut self) -> Result<Option<Message>> {
        // Read first 2 bytes (FIN + opcode, MASK + payload length)
        let mut header = [0u8; 2];
        if self.stream.read_exact(&mut header).is_err() {
            return Ok(None); // Connection closed
        }

        let _fin = header[0] & 0x80 != 0;
        let opcode = header[0] & 0x0F;
        let masked = header[1] & 0x80 != 0;
        let mut payload_len = (header[1] & 0x7F) as u64;

        // Extended payload length
        if payload_len == 126 {
            let mut buf = [0u8; 2];
            self.stream
                .read_exact(&mut buf)
                .map_err(|e| Error::WebSocket(e.to_string()))?;
            payload_len = u16::from_be_bytes(buf) as u64;
        } else if payload_len == 127 {
            let mut buf = [0u8; 8];
            self.stream
                .read_exact(&mut buf)
                .map_err(|e| Error::WebSocket(e.to_string()))?;
            payload_len = u64::from_be_bytes(buf);
        }

        // Read masking key (4 bytes, if present)
        let mask_key = if masked {
            let mut key = [0u8; 4];
            self.stream
                .read_exact(&mut key)
                .map_err(|e| Error::WebSocket(e.to_string()))?;
            Some(key)
        } else {
            None
        };

        // Read payload
        let mut payload = vec![0u8; payload_len as usize];
        if payload_len > 0 {
            self.stream
                .read_exact(&mut payload)
                .map_err(|e| Error::WebSocket(e.to_string()))?;
        }

        // Unmask payload
        if let Some(key) = mask_key {
            for (i, byte) in payload.iter_mut().enumerate() {
                *byte ^= key[i % 4];
            }
        }

        // Decode based on opcode
        match Opcode::from_u8(opcode) {
            Some(Opcode::Text) => {
                let text = String::from_utf8_lossy(&payload).to_string();
                Ok(Some(Message::Text(text)))
            }
            Some(Opcode::Binary) => Ok(Some(Message::Binary(payload))),
            Some(Opcode::Close) => {
                // Send close frame back
                let _ = self.write_frame(Opcode::Close as u8, &[]);
                Ok(Some(Message::Close))
            }
            Some(Opcode::Ping) => {
                // Auto-respond with pong
                let _ = self.write_frame(Opcode::Pong as u8, &payload);
                Ok(Some(Message::Ping(payload)))
            }
            Some(Opcode::Pong) => Ok(Some(Message::Pong(payload))),
            Some(Opcode::Continue) => Ok(None), // Simplified: ignore continuation frames
            None => Err(Error::WebSocket(format!("Unknown opcode: {}", opcode))),
        }
    }

    /// Send a text message.
    pub fn send_text(&mut self, text: &str) -> Result<()> {
        self.write_frame(Opcode::Text as u8, text.as_bytes())
    }

    /// Send a binary message.
    pub fn send_binary(&mut self, data: &[u8]) -> Result<()> {
        self.write_frame(Opcode::Binary as u8, data)
    }

    /// Send a ping message.
    pub fn send_ping(&mut self, data: &[u8]) -> Result<()> {
        self.write_frame(Opcode::Ping as u8, data)
    }

    /// Send a close message and close the connection.
    pub fn close(&mut self) -> Result<()> {
        self.write_frame(Opcode::Close as u8, &[])
    }

    /// Write a WebSocket frame (server frames are NOT masked per RFC 6455).
    fn write_frame(&mut self, opcode: u8, payload: &[u8]) -> Result<()> {
        let mut frame = Vec::new();

        // FIN bit + opcode
        frame.push(0x80 | opcode);

        // Payload length (server -> client: no masking)
        let len = payload.len();
        if len < 126 {
            frame.push(len as u8);
        } else if len <= 65535 {
            frame.push(126);
            frame.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            frame.push(127);
            frame.extend_from_slice(&(len as u64).to_be_bytes());
        }

        frame.extend_from_slice(payload);

        self.stream
            .write_all(&frame)
            .map_err(|e| Error::WebSocket(e.to_string()))?;
        self.stream
            .flush()
            .map_err(|e| Error::WebSocket(e.to_string()))?;

        Ok(())
    }
}
