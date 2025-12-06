use std::sync::OnceLock;

static SERVER_ADDR: OnceLock<String> = OnceLock::new();
static WORKER_COUNT: OnceLock<usize> = OnceLock::new();

pub const HEADER_SIZE: usize = 5;

pub fn get_server_addr() -> &'static str {
    SERVER_ADDR.get_or_init(|| "127.0.0.1:9000".to_string())
}

pub fn get_worker_count() -> usize {
    *WORKER_COUNT.get_or_init(|| 4)
}

pub fn set_server_addr(addr: String) {
    SERVER_ADDR.set(addr).ok();
}

pub fn set_worker_count(count: usize) {
    WORKER_COUNT.set(count).ok();
}