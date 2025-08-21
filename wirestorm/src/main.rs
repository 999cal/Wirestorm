//! WireStorm CTMP Proxy (Part 1)
//!
//! This Rust program implements a simple CoreTech Message Protocol (CTMP) proxy.
//! It listens for a single source client on port 33333 and multiple destination
//! clients on port 44444. Messages from the source are parsed and then
//! broadcasted to all connected destination clients. Invalid messages or
//! failed writes result in the corresponding client being disconnected.

use std::{
    net::{TcpListener, TcpStream}, // For TCP network communication
    sync::{Arc, Mutex},            // For thread-safe shared state
    thread,                        // For multithreading
    io::Write,                     // For writing bytes to TCP streams
};

mod ctmp; // Module handling CTMP message parsing

fn main() {
    // Shared list of connected destination clients, wrapped in Arc<Mutex<>> for safe concurrent access
    let dest_clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    // Destination listener setup (port 44444)
    {
        // Clone Arc pointer for use inside the thread
        let dest_clients = Arc::clone(&dest_clients);

        // Spawn a thread to accept destination client connections
        thread::spawn(move || {
            // Bind TCP listener to all interfaces on port 44444
            let listener = TcpListener::bind("0.0.0.0:44444").expect("Failed to bind 44444");
            println!("Listening for destination clients on 44444...");

            // Accept incoming connections in a loop
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    // Print client address if available
                    if let Ok(addr) = stream.peer_addr() {
                        println!("Destination client connected: {}", addr);
                    } else {
                        println!("Destination client connected (unknown addr)");
                    }

                    // Lock the shared destination client list and add the new client
                    if let Ok(mut clients) = dest_clients.lock() {
                        clients.push(stream);
                    } else {
                        // If mutex is poisoned, log error
                        eprintln!("Mutex poisoned while adding destination client");
                    }
                }
            }
        });
    }

    // Source listener setup (port 33333)
    let listener = TcpListener::bind("0.0.0.0:33333").expect("Failed to bind 33333");
    println!("Waiting for source clients on port 33333...");

    // Accept incoming source client connections
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            // Print the address of the connected source client
            if let Ok(addr) = stream.peer_addr() {
                println!("Source connected from {}", addr);
            }

            // Clone Arc pointer to share the destination client list with the new thread
            let dest_clients = Arc::clone(&dest_clients);

            // Spawn a thread to handle communication with this source client
            thread::spawn(move || {
                let mut stream = stream;

                loop {
                    // Parse CTMP messages from the source client
                    match ctmp::parse_ctmp_message(&mut stream) {
                        Ok(Some(message)) => {
                            // Successfully parsed a message; broadcast to all destination clients
                            if let Ok(mut clients) = dest_clients.lock() {
                                // Retain only clients that successfully receive the message
                                clients.retain_mut(|client| {
                                    if let Err(e) = client.write_all(&message) {
                                        // If write fails, remove the client and log the error
                                        if let Ok(addr) = client.peer_addr() {
                                            println!("Dropping client ({}): {}", addr, e);
                                        } else {
                                            println!("Dropping client (unknown addr): {}", e);
                                        }
                                        return false; // Remove client from list
                                    }
                                    true // Keep client in list
                                });
                            } else {
                                // Mutex poisoned, log and exit the thread
                                eprintln!("Mutex poisoned while broadcasting");
                                break;
                            }
                        }
                        Ok(None) => {
                            // End-of-stream detected; disconnect source
                            break;
                        }
                        Err(e) => {
                            // Error while reading or parsing; log and disconnect source
                            println!("Error reading from source: {}", e);
                            break;
                        }
                    }
                }
            });
        }
    }
}
