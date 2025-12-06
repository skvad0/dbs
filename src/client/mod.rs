use std::fs;
use std::io::{self, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::utils::protocol::{Message, OpCode};

// Client that submits files to server for compilation
pub fn submit_files(files: Vec<String>, server_addr: &str) -> io::Result<()> {
    println!("[Client] Connecting to build server at {}", server_addr);
    println!("[Client] Submitting {} files in parallel...", files.len());
    
    let results: Arc<Mutex<Vec<(String, bool)>>> = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();
    
    for file_path in files {
        let server_addr = server_addr.to_string();
        let results_clone = Arc::clone(&results);
        
        let handle = thread::spawn(move || {
            if let Err(e) = submit_single_file(&file_path, &server_addr) {
                eprintln!("[Client] Error submitting {}: {}", file_path, e);
                let mut r = results_clone.lock().unwrap();
                r.push((file_path, false));
            } else {
                let mut r = results_clone.lock().unwrap();
                r.push((file_path, true));
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all submissions to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Report summary
    let final_results = results.lock().unwrap();
    let success_count = final_results.iter().filter(|(_, success)| *success).count();
    println!("\n[Client] Submission complete: {}/{} files succeeded.", success_count, final_results.len());
    
    Ok(())
}

fn submit_single_file(file_path: &str, server_addr: &str) -> io::Result<()> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    }
    
    if !file_path.ends_with(".c") {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not a .c file"));
    }
    
    println!("[Client] Submitting {}...", file_path);
    
    // Create connection for this file
    let mut stream = TcpStream::connect(server_addr)?;
    
    // Read file contents
    let file_contents = fs::read(file_path)?;
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.c");
    
    // Create payload: [filename_len (4 bytes)][filename][file_contents]
    let mut payload = Vec::new();
    let filename_bytes = filename.as_bytes();
    payload.extend_from_slice(&(filename_bytes.len() as u32).to_be_bytes());
    payload.extend_from_slice(filename_bytes);
    payload.extend_from_slice(&file_contents);
    
    // Send SubmitFile message
    let msg = Message::new(OpCode::SubmitFile, payload);
    stream.write_all(&msg.serialize())?;
    
    // Wait for FileResult
    let result = Message::read(&mut stream)?;
    
    if result.op == OpCode::FileResult {
        // Parse: [1 byte success][4 bytes filename_len][filename][.o file contents or error msg]
        if result.payload.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Empty response from server"));
        }
        
        let success = result.payload[0] == 1;
        let filename_len = u32::from_be_bytes(result.payload[1..5].try_into().unwrap()) as usize;
        let returned_filename = String::from_utf8_lossy(&result.payload[5..5+filename_len]).to_string();
        let data = &result.payload[5+filename_len..];
        
        if success {
            // Save .o file
            let output_path = file_path.replace(".c", ".o");
            fs::write(&output_path, data)?;
            println!("[Client] Received: {} -> {}", returned_filename, output_path);
        } else {
            // Error message
            let error_msg = String::from_utf8_lossy(data);
            eprintln!("[Client] Compilation failed for {}: {}", returned_filename, error_msg);
            return Err(io::Error::new(io::ErrorKind::Other, format!("Compilation failed: {}", error_msg)));
        }
    } else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpected response from server"));
    }
    
    Ok(())
}
