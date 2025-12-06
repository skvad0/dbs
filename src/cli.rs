use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dbs")]
#[command(version = "0.1.0")]
#[command(about = "Distributed Build System - Compile C files in parallel across worker nodes", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the controller node and build C files locally
    Build {
        /// C source files to compile
        #[arg(required = true)]
        files: Vec<String>,

        /// Number of worker processes to spawn
        #[arg(short, long, default_value_t = 4)]
        workers: usize,

        /// Server address for workers to connect to
        #[arg(short, long, default_value = "127.0.0.1:9000")]
        address: String,
    },

    /// Start a server that accepts file submissions from clients
    Serve {
        /// Number of worker processes to spawn
        #[arg(short, long, default_value_t = 4)]
        workers: usize,

        /// Server address to bind to
        #[arg(short, long, default_value = "127.0.0.1:9000")]
        address: String,
    },

    /// Submit C files to a remote build server for compilation
    Submit {
        /// C source files to submit
        #[arg(required = true)]
        files: Vec<String>,

        /// Build server address
        #[arg(short, long, default_value = "127.0.0.1:9000")]
        server: String,
    },

    /// Start a worker node (internal use)
    #[command(hide = true)]
    Worker {
        /// Worker ID
        id: String,
    },
}
