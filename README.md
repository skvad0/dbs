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

Compile C files using local workers:

```bash
dbs build file1.c file2.c file3.c --workers 4
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

Submit files to a remote build server:

```bash
# Submit to localhost
dbs submit file1.c file2.c file3.c

# Submit to remote server
dbs submit file1.c file2.c --server 192.168.1.100:9000
```


