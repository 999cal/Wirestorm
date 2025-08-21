//! CTMP Message Parser with Checksum Support
//!
//! This module provides a function to parse CoreTech Message Protocol (CTMP) messages
//! from a TCP stream. Each message consists of an 8-byte header followed by a payload.
//! If the message is marked as "sensitive" (bit 6 of the options byte), a 16-bit one's
//! complement checksum is validated. Invalid messages are dropped. The parser returns
//! the full message (header + payload) as a vector of bytes.

use std::io::{self, Read}; // For reading from TCP streams
use std::net::TcpStream;   // TCP stream type

/// Compute 16-bit one's complement checksum over the provided buffer.
///
/// - Sum 16-bit words in big-endian order
/// - If buffer has odd length, pad last byte with 0
/// - Fold sum into 16 bits and return one's complement
fn compute_checksum(buf: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut chunks = buf.chunks_exact(2);

    // Sum all 16-bit words
    for chunk in &mut chunks {
        let word = u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
        sum = sum.wrapping_add(word);
    }

    // Handle any remaining single byte (pad with 0)
    if let [last] = chunks.remainder() {
        let word = (*last as u32) << 8;
        sum = sum.wrapping_add(word);
    }

    // Fold carry bits into 16 bits
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !(sum as u16) // Return one's complement
}

/// Parses a single CTMP message from the TCP stream.
///
/// Returns:
/// - `Ok(Some(Vec<u8>))` if a full, valid message was read
/// - `Ok(None)` if the stream closed or the message is invalid
/// - `Err(io::Error)` if an unexpected IO error occurs
pub fn parse_ctmp_message(stream: &mut TcpStream) -> io::Result<Option<Vec<u8>>> {
    // Allocate buffer for 8-byte header
    let mut header = [0u8; 8];

    // Attempt to read exactly 8 bytes for header
    if stream.read_exact(&mut header).is_err() {
        return Ok(None); // Stream closed or error: treat as disconnect
    }

    // Validate "magic" byte to confirm it's a CTMP message
    if header[0] != 0xCC {
        return Ok(None); // Not a valid message
    }

    let options = header[1];                             // Options / flags byte
    let length = u16::from_be_bytes([header[2], header[3]]) as usize; // Payload length
    let checksum_field = u16::from_be_bytes([header[4], header[5]]); // Provided checksum
    // header[6..8] = padding (ignored)

    // Read payload of `length` bytes
    let mut data = vec![0u8; length];
    if stream.read_exact(&mut data).is_err() {
        return Ok(None); // Stream closed unexpectedly
    }

    // If message is sensitive (bit 6 of options), validate checksum
    if (options & 0b0100_0000) != 0 {
        // Build a buffer of header + payload with checksum bytes set to 0xCCCC
        let mut checksum_buf = header.to_vec();
        checksum_buf[4] = 0xCC;
        checksum_buf[5] = 0xCC;
        checksum_buf.extend_from_slice(&data);

        let calc = compute_checksum(&checksum_buf); // Compute checksum

        if calc != checksum_field {
            eprintln!("Dropping message due to invalid checksum");
            return Ok(None); // Drop invalid message
        }
    }

    // Build full message buffer (header + payload)
    let mut message = Vec::with_capacity(8 + length);
    message.extend_from_slice(&header);
    message.extend_from_slice(&data);

    Ok(Some(message)) // Return the complete CTMP message
}
