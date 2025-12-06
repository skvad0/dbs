mod cli;
mod client;
mod server;
mod utils;
mod worker;

use clap::Parser;
use cli::{Cli, Commands};
use client::submit_files;
use server::server_node;
use utils::config;
use worker::{controller::controller_node, worker_node};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            files,
            workers,
            address,
        } => {
            config::set_worker_count(workers);
            config::set_server_addr(address);
            
            controller_node(files);
        }
        Commands::Serve { workers, address } => {
            config::set_worker_count(workers);
            config::set_server_addr(address);
            
            server_node();
        }
        Commands::Submit { files, server } => {
            if let Err(e) = submit_files(files, &server) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Worker { id } => {
            worker_node(&id);
        }
    }
}
