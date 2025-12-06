
pub fn determine_workload(files: Vec<String>) -> Vec<String> {
    if files.is_empty() {
        eprintln!("Error: No C files provided as arguments.");
        eprintln!("Usage: dbs build <file1.c> <file2.c> ...");
        eprintln!("Example: dbs build (Get-ChildItem *.c).FullName");
        std::process::exit(1);
    }

    println!("[Cluster] Using provided files as workload.");
    files
}

pub fn validate_worker_count(requested: usize) -> usize {
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    
    if requested > cpu_count * 2 {
        println!(
            "[Warning] {} workers requested, but only {} CPU cores detected.",
            requested, cpu_count
        );
        println!("[Warning] Limiting to {} workers (2x CPU cores) for optimal performance.", cpu_count * 2);
        cpu_count * 2
    } else {
        requested
    }
}
