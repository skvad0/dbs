pub mod controller;
pub(crate) mod workload;

use std::io::Write;
use std::net::TcpStream;
use std::process::Command;
use std::thread;

use crate::utils::config;
use crate::utils::protocol::{Message, OpCode};

pub fn worker_node(id: &str) {
    let server_addr = config::get_server_addr();
    
    let mut stream = loop {
        match TcpStream::connect(server_addr) {
            Ok(s) => break s,
            Err(_) => thread::sleep(std::time::Duration::from_millis(100)),
        }
    };

    let hello = Message::new(OpCode::Hello, format!("Worker-{}", id).into_bytes());
    stream.write_all(&hello.serialize()).unwrap();

    while let Ok(msg) = Message::read(&mut stream) {

        match msg.op {
            OpCode::TaskDef => {
                // Payload is the file path string
                let path = String::from_utf8_lossy(&msg.payload).to_string();

                println!("\t[Worker #{}] Compiling {}...", id, path);

                // EXECUTE GCC
                // gcc -c path/to/file.c -o path/to/file.o
                let output_file = path.replace(".c", ".o");
                let output = Command::new("gcc")
                    .arg("-c")
                    .arg(&path)
                    .arg("-o")
                    .arg(&output_file)
                    .output();

                let (success, log) = match output {
                    Ok(out) => {
                        let log = if out.status.success() {
                            "OK".to_string()
                        } else {
                            String::from_utf8_lossy(&out.stderr).to_string()
                        };
                        (out.status.success(), log)
                    }
                    Err(e) => (false, e.to_string()), // GCC likely not found
                };

                // Serialize: [1 byte bool] [Output String]
                let mut resp_payload = Vec::new();
                resp_payload.push(if success { 1 } else { 0 });
                resp_payload.extend_from_slice(log.as_bytes());

                let resp = Message::new(OpCode::TaskResult, resp_payload);
                stream.write_all(&resp.serialize()).unwrap();
            }
            OpCode::Shutdown => break,
            _ => {}
        }
    }
}
