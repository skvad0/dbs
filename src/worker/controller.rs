use std::env;
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::utils::config;
use crate::server::session::handle_worker_session;
use super::workload::{determine_workload, validate_worker_count};

pub fn controller_node(files: Vec<String>) {
    let server_addr = config::get_server_addr();
    let worker_count = validate_worker_count(config::get_worker_count());
    
    println!("[Cluster] Starting Build Server on {}", server_addr);
    println!("[Cluster] Using {} worker processes", worker_count);

    let workload = determine_workload(files);
    let total_tasks = workload.len();
    let queue = Arc::new(Mutex::new(workload));
    // Store results as (Filename, Success_Bool, Message)
    let results = Arc::new(Mutex::new(Vec::new()));

    let listener = TcpListener::bind(server_addr).expect("Bind failed");

    // 2. Spawn Worker Processes
    let current_exe = env::current_exe().unwrap();
    let mut children = Vec::new();

    for i in 0..worker_count {
        println!("[Cluster] Booting Worker Process #{}", i);
        let child = Command::new(&current_exe)
            .arg("worker")
            .arg(i.to_string())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to spawn worker");
        children.push(child);
    }

    // 3. Connection Handling Loop
    let mut handles = Vec::new();
    println!("[Cluster] Waiting for workers to connect...");

    for _ in 0..worker_count {
        let (stream, addr) = listener.accept().unwrap();
        println!("[Cluster] Worker connected from {}", addr);

        let q_clone = Arc::clone(&queue);
        let r_clone = Arc::clone(&results);

        let handle = thread::spawn(move || {
            handle_worker_session(stream, q_clone, r_clone);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }
    for mut child in children {
        child.wait().unwrap();
    }

    // 6. Report
    let final_results = results.lock().unwrap();
    println!("\n=== BUILD REPORT ===");
    let success_count = final_results.iter().filter(|r| r.1).count();
    println!(
        "Build Complete: {}/{} Succeeded.",
        success_count, total_tasks
    );

    if success_count == total_tasks {
        println!("All files compiled successfully to .o files.");
    } else {
        println!("Some files failed. Check stdout for details.");
    }
}
