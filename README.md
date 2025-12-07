# Distributed Build System

A prototype for a distributed build system written in Rust.

## Description

This project is an experiment in creating a system that can distribute compilation tasks across multiple networked machines to speed up build times. The C files in the `/examples` directory are used as sample data for compilation.

## Architecture

It uses a controller-worker model:
- **Controller**: (`src/controller.rs`) - Listens for work and dispatches tasks to available workers.
- **Worker**: (`src/worker.rs`) - Receives tasks, executes them, and reports back the results.
- **Protocol**: (`src/protocol.rs`) - Defines the network protocol for communication between the controller and workers.

## How to Run

### Build the project

```bash
cargo build --release
```

### Local Build Mode

Compile C files using local workers (use full or relative paths to .c files):

```bash
# Using relative paths
dbs build file1.c file2.c file3.c --workers 4

# Using absolute paths
dbs build C:/projects/myapp/file1.c C:/projects/myapp/file2.c --workers 4

# Using wildcards
dbs build src/*.c --workers 4
```

### Server Mode

Start a server that accepts client file submissions:

```bash
# Local network only
dbs serve --workers 4

# Accessible from other machines (use 0.0.0.0)
dbs serve --workers 4 --address 0.0.0.0:9000
```

### Client Mode

Submit files to a remote build server (use full or relative paths to .c files):

```bash
# Submit to localhost
dbs submit /path/to/file1.c /path/to/file2.c /path/to/file3.c

# Submit to remote server (replace IP with your server's actual IP address)
# The IP address will be different depending on your network
# Use ipconfig (Windows) or ifconfig (Linux/Mac) to find your server's IP
dbs submit /path/to/file1.c /path/to/file2.c --server 192.168.1.100:9000

# Example: Submit with absolute paths to a specific server
dbs submit C:/Users/YourName/project/main.c C:/Users/YourName/project/utils.c --server 10.0.0.5:9000
```

**Note:** The server IP address (e.g., `192.168.1.100`) is just an example. Replace it with:
- Your server machine's actual local IP address (for LAN)
- Your public IP address (for internet access)
- Your VPN IP address (if using VPN like Tailscale)

To find your server's IP address:
- **Windows**: Run `ipconfig` and look for IPv4 Address
- **Linux/Mac**: Run `ifconfig` or `ip addr`


