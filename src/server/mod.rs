mod client_handler;
pub mod session;

use std::env;
use std::net::TcpListener;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use client_handler::handle_client_session;
use session::handle_worker_session;

use crate::utils::config;

// Server that accepts client file submissions
pub fn server_node() {
    let server_addr = config::get_server_addr();
    
    println!("[Server] Starting file submission server on {}", server_addr);
    println!("[Server] Clients can submit files for compilation");
    
    let listener = TcpListener::bind(server_addr).expect("Bind failed");
    
    // Shared queue and results (empty initially, filled by clients)
    let queue: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let results: Arc<Mutex<Vec<(String, bool, String)>>> = Arc::new(Mutex::new(Vec::new()));
    
    // Spawn local workers
    let worker_count = config::get_worker_count();
    let current_exe = env::current_exe().unwrap();
    let mut _children = Vec::new();
    
    for i in 0..worker_count {
        println!("[Server] Starting worker #{}", i);
        let child = Command::new(&current_exe)
            .arg("worker")
            .arg(i.to_string())
            .spawn()
            .expect("Failed to spawn worker");
        _children.push(child);
    }
    
    // Accept worker connections first
    println!("[Server] Waiting for {} workers to connect...", worker_count);
    let mut worker_handles = Vec::new();
    
    for _ in 0..worker_count {
        let (stream, addr) = listener.accept().unwrap();
        println!("[Server] Worker connected from {}", addr);
        
        let q = Arc::clone(&queue);
        let r = Arc::clone(&results);
        
        let handle = thread::spawn(move || {
            handle_worker_session(stream, q, r);
        });
        worker_handles.push(handle);
    }
    
    println!("[Server] All workers connected. Ready to accept client submissions.");
    
    // Accept client connections
    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                println!("[Server] Client connected from {}", addr);
                
                let q = Arc::clone(&queue);
                let r = Arc::clone(&results);
                
                thread::spawn(move || {
                    if let Err(e) = handle_client_session(stream, q, r) {
                        eprintln!("[Server] Client error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("[Server] Connection error: {}", e);
            }
        }
    }
}
