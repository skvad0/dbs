use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::utils::protocol::{Message, OpCode};

// Handle communication with a single worker
pub fn handle_worker_session(
    mut stream: TcpStream,
    queue: Arc<Mutex<Vec<String>>>,
    results: Arc<Mutex<Vec<(String, bool, String)>>>,
) {
    if let Ok(msg) = Message::read(&mut stream) {
        if msg.op != OpCode::Hello {
            return;
        }
    }

    loop {
        let task_opt = {
            let mut q = queue.lock().unwrap();
            q.pop()
        };

        match task_opt {
            Some(filepath) => {
                // Send Filename as bytes
                let req = Message::new(OpCode::TaskDef, filepath.clone().into_bytes());
                if stream.write_all(&req.serialize()).is_err() {
                    break;
                }

                // Wait for Result
                let res_msg = match Message::read(&mut stream) {
                    Ok(m) => m,
                    Err(_) => break,
                };

                if res_msg.op == OpCode::TaskResult {
                    // Protocol: [1 byte status] [rest is msg string]
                    if !res_msg.payload.is_empty() {
                        let success = res_msg.payload[0] == 1;
                        let out_msg = String::from_utf8_lossy(&res_msg.payload[1..]).to_string();

                        let mut r_guard = results.lock().unwrap();
                        r_guard.push((filepath, success, out_msg));
                    }
                }
            }
            None => {
                // Queue is empty, wait a bit and try again
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}
