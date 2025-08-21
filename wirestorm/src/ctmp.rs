//! CTMP Message Parser
//!
//! This module provides a function to parse CoreTech Message Protocol (CTMP) messages
//! from a TCP stream. Each message consists of an 8-byte header followed by a payload.
//! The parser validates the header, reads the payload, and returns the complete message
//! as a vector of bytes. If the connection closes gracefully, it returns `None`.

use std::io::{self, Read}; // For reading bytes from streams
use std::net::TcpStream;   // For TCP stream handling

/// Parses a single CTMP message from the given TCP stream.
/// 
/// Returns:
/// - `Ok(Some(Vec<u8>))` if a full message was successfully read,
/// - `Ok(None)` if the stream closed gracefully or the header is invalid,
/// - `Err(io::Error)` if an unexpected IO error occurs.
pub fn parse_ctmp_message(stream: &mut TcpStream) -> io::Result<Option<Vec<u8>>> {
    let mut header = [0u8; 8]; // Allocate buffer for 8-byte CTMP header

    // Try to read exactly 8 bytes from the stream
    if let Err(e) = stream.read_exact(&mut header) {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            return Ok(None); // Connection closed gracefully
        }
        return Err(e); // Unexpected IO error, propagate it
    }

    // Validate header fields according to CTMP protocol
    if header[0] != 0xCC {
        return Ok(None); // Invalid message start byte
    }
    if header[1] != 0x00 {
        return Ok(None); // Invalid version or reserved byte
    }
    if header[4..8] != [0x00, 0x00, 0x00, 0x00] {
        return Ok(None); // Reserved bytes must be zero
    }

    // LENGTH field (2 bytes, big endian) is at header[2..4]
    let length = u16::from_be_bytes([header[2], header[3]]) as usize;

    // Read payload of specified length
    let mut data = vec![0u8; length];
    stream.read_exact(&mut data)?; // May return Err if stream closes unexpectedly

    // Combine header and payload into a single message vector
    let mut message = Vec::with_capacity(8 + length); // Pre-allocate to avoid resizing
    message.extend_from_slice(&header); // Add header first
    message.extend_from_slice(&data);   // Append payload

    Ok(Some(message)) // Return full CTMP message
}
