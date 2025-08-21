//! CTMP TCP Proxy
//!
//! This program acts as a TCP proxy for the CoreTech Message Protocol (CTMP).
//! It listens on two ports:
//! - 33333: Source clients (send messages to the proxy)
//! - 44444: Destination clients (receive messages from all sources)
//!
//! Each source connection is handled in its own thread. Messages are parsed using
//! `ctmp::parse_ctmp_message` and broadcast to all connected destinations. Destination
//! clients are also handled in separate threads to maintain the connection and remove
//! disconnected clients.

use std::io::{Read, Write};       // For reading/writing to TCP streams
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};      // Thread-safe shared vector for destinations
use std::thread;

mod ctmp;

/// Handles a source client.
/// Reads CTMP messages from the source and broadcasts them to all destinations.
fn handle_source(mut stream: TcpStream, destinations: Arc<Mutex<Vec<TcpStream>>>) {
    loop {
        match ctmp::parse_ctmp_message(&mut stream) {
            Ok(Some(message)) => {
                // Lock the destinations list for writing
                let mut destinations = destinations.lock().unwrap();

                // Retain only clients that successfully receive the message
                destinations.retain_mut(|dest| {
                    if let Err(e) = dest.write_all(&message) {
                        eprintln!("Destination write failed: {}", e);
                        false // drop disconnected client
                    } else {
                        true
                    }
                });
            }
            Ok(None) => {
                eprintln!("Source disconnected or message dropped.");
                break; // Exit loop if source disconnected or invalid message
            }
            Err(e) => {
                eprintln!("Error reading source message: {}", e);
                break; // Exit loop on read error
            }
        }
    }
}

/// Handles a destination client.
/// Adds the destination to the shared list and keeps the connection alive.
fn handle_destination(mut stream: TcpStream, destinations: Arc<Mutex<Vec<TcpStream>>>) {
    {
        // Add destination client to shared list
        let mut dests = destinations.lock().unwrap();
        dests.push(stream.try_clone().expect("Failed to clone destination"));
    }

    // Keep the connection alive until the client disconnects
    let mut buf = [0u8; 1];
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            break; // Client disconnected
        }
    }

    eprintln!("Destination disconnected.");

    // Remove any disconnected destinations
    let mut dests = destinations.lock().unwrap();
    dests.retain(|s| s.peer_addr().is_ok());
}

fn main() -> std::io::Result<()> {
    // Listen for source connections
    let sources = TcpListener::bind("0.0.0.0:33333")?;
    // Listen for destination connections
    let destinations = TcpListener::bind("0.0.0.0:44444")?;

    // Shared list of destination clients
    let destinations_list: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    // Spawn a thread to handle incoming source connections
    {
        let destinations_list = Arc::clone(&destinations_list);
        thread::spawn(move || {
            println!("Waiting for source clients on port 33333...");
            for stream in sources.incoming() {
                match stream {
                    Ok(stream) => {
                        println!("Source connected from {}", stream.peer_addr().unwrap());
                        let dests = Arc::clone(&destinations_list);
                        // Spawn a thread to handle this source
                        thread::spawn(move || handle_source(stream, dests));
                    }
                    Err(e) => eprintln!("Source connection failed: {}", e),
                }
            }
        });
    }

    // Accept destination connections in the main thread
    println!("Listening for destination clients on 44444...");
    for stream in destinations.incoming() {
        match stream {
            Ok(stream) => {
                println!("Destination client connected: {}", stream.peer_addr().unwrap());
                let dests = Arc::clone(&destinations_list);
                // Spawn a thread to handle this destination
                thread::spawn(move || handle_destination(stream, dests));
            }
            Err(e) => eprintln!("Destination connection failed: {}", e),
        }
    }

    Ok(())
}
