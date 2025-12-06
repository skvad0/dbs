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

*(TODO: Add instructions on how to start the controller and worker(s) and how to submit a build job.)*
