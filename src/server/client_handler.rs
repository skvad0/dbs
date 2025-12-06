use std::fs;
use std::io::{self, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::utils::protocol::{Message, OpCode};

// Handle a client connection that submits files for compilation
pub fn handle_client_session(
    mut stream: TcpStream,
    queue: Arc<Mutex<Vec<String>>>,
    results: Arc<Mutex<Vec<(String, bool, String)>>>,
) -> io::Result<()> {
    // Receive SubmitFile message
    let msg = Message::read(&mut stream)?;
    
    if msg.op != OpCode::SubmitFile {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Expected SubmitFile message",
        ));
    }
    
    // Parse payload: [filename_len (4 bytes)][filename][file_contents]
    if msg.payload.len() < 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid payload format",
        ));
    }
    
    let filename_len = u32::from_be_bytes(msg.payload[0..4].try_into().unwrap()) as usize;
    if msg.payload.len() < 4 + filename_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid filename length",
        ));
    }
    
    let filename = String::from_utf8_lossy(&msg.payload[4..4 + filename_len]).to_string();
    let file_contents = &msg.payload[4 + filename_len..];
    
    println!("[Server] Client submitted: {}", filename);
    
    // Save file temporarily
    let temp_dir = PathBuf::from("temp_builds");
    fs::create_dir_all(&temp_dir)?;
    
    let temp_file_path = temp_dir.join(&filename);
    fs::write(&temp_file_path, file_contents)?;
    
    let temp_file_str = temp_file_path.to_string_lossy().to_string();
    
    // Add to build queue
    {
        let mut q = queue.lock().unwrap();
        q.push(temp_file_str.clone());
        println!("[Server] Added {} to queue. Queue size: {}", filename, q.len());
    }
    
    println!("[Server] Waiting for worker to compile {}...", filename);
    
    // Wait for compilation to complete
    // Poll results until our file appears
    let output_file = temp_file_path.to_string_lossy().replace(".c", ".o");
    
    let mut timeout_counter = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        timeout_counter += 1;
        
        if timeout_counter > 100 { // 10 second timeout
            println!("[Server] Timeout waiting for compilation of {}", filename);
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Compilation timeout",
            ));
        }
        
        let results_guard = results.lock().unwrap();
        if let Some((_, success, log)) = results_guard
            .iter()
            .find(|(path, _, _)| path == &temp_file_str)
        {
            // Build complete! Send result back
            let mut response_payload = Vec::new();
            
            if *success {
                // Read .o file
                match fs::read(&output_file) {
                    Ok(obj_contents) => {
                        response_payload.push(1); // Success
                        let output_filename = filename.replace(".c", ".o");
                        let output_filename_bytes = output_filename.as_bytes();
                        response_payload.extend_from_slice(&(output_filename_bytes.len() as u32).to_be_bytes());
                        response_payload.extend_from_slice(output_filename_bytes);
                        response_payload.extend_from_slice(&obj_contents);
                        
                        println!("[Server] Sending compiled .o file for {} back to client", filename);
                    }
                    Err(e) => {
                        response_payload.push(0); // Failure
                        let error_msg = format!("Failed to read .o file: {}", e);
                        let filename_bytes = filename.as_bytes();
                        response_payload.extend_from_slice(&(filename_bytes.len() as u32).to_be_bytes());
                        response_payload.extend_from_slice(filename_bytes);
                        response_payload.extend_from_slice(error_msg.as_bytes());
                    }
                }
            } else {
                // Compilation failed
                response_payload.push(0); // Failure
                let filename_bytes = filename.as_bytes();
                response_payload.extend_from_slice(&(filename_bytes.len() as u32).to_be_bytes());
                response_payload.extend_from_slice(filename_bytes);
                response_payload.extend_from_slice(log.as_bytes());
            }
            
            let response = Message::new(OpCode::FileResult, response_payload);
            stream.write_all(&response.serialize())?;
            
            // Cleanup
            fs::remove_file(&temp_file_path).ok();
            if *success {
                fs::remove_file(&output_file).ok();
            }
            
            break;
        }
        drop(results_guard);
    }
    
    Ok(())
}
