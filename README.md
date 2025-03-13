# Succinct Sudoku Backend

A Rust-based backend service that provides zero-knowledge proof (ZKP) verification for Sudoku puzzles using the SP1 ZK-VM framework. This system allows users to submit Sudoku puzzles and their solutions, then generates cryptographic proofs that verify the solution is correct without revealing the solution itself.

## 🚀 Features

- **Sudoku Verification**: Verifies that Sudoku solutions are valid according to standard rules
- **Zero-Knowledge Proofs**: Generates cryptographic proofs using SP1 framework
- **Asynchronous Job Processing**: Handles proof generation as asynchronous jobs
- **WebSocket Support**: Provides real-time updates on job status and logs
- **RESTful API**: Clean API for submitting puzzles and retrieving proof status

## 🔧 Technical Architecture

### Core Technologies

- **Rust**: The entire backend is written in Rust, leveraging its safety and performance features
- **Axum Framework**: Modern, ergonomic web framework for building HTTP and WebSocket servers
- **Tokio**: Asynchronous runtime for handling concurrent operations
- **SP1 ZK-VM**: Zero-knowledge virtual machine framework by Succinct Labs
- **Serde**: For serialization and deserialization of JSON data

### System Components

#### Data Models

- `SudokuBoard`: Represents a Sudoku puzzle with a 9x9 grid
- `SudokuSolution`: Represents a solution to a Sudoku puzzle
- `ProofInput`: Contains both the initial board and solution
- `ProofResponse`: Contains the generated proof and public values
- `JobResponse`: Represents the status and result of a proof generation job

#### Server Implementation

The main server is implemented in `src/main.rs` and provides:
- HTTP endpoints for submitting puzzles and checking proof status
- WebSocket connections for real-time updates
- Middleware for logging and CORS
- Job storage and processing management

#### SP1 Prover

The SP1 prover (`sp1_prover/src/main.rs`) implements:
- Verification logic that runs inside the SP1 ZK-VM
- Sudoku solution validation according to standard rules
- Cryptographic proof generation

#### Verification Logic

Core verification functions in `src/lib.rs`:
- `is_valid_sudoku`: Checks if a Sudoku board follows the rules
- `verify_solution`: Verifies that a solution is valid and matches the initial board

## 📡 API Endpoints

### HTTP Endpoints

- `POST /api/prove`: Submit a Sudoku puzzle and solution for verification
  ```json
  {
    "board": [[0,0,0,0,0,0,0,0,0], ...],
    "solution": [[1,2,3,4,5,6,7,8,9], ...]
  }
  ```

- `GET /api/status/:job_id`: Check the status of a proof generation job
  ```json
  {
    "job_id": "123e4567-e89b-12d3-a456-426614174000",
    "status": "Complete",
    "result": {
      "public_values": "...",
      "proof": "..."
    },
    "error": null
  }
  ```

### WebSocket Endpoints

- `GET /api/ws/proof/:job_id`: WebSocket connection for real-time proof updates
- `GET /api/ws/logs/:job_id`: WebSocket connection for real-time log updates

## 🔐 Zero-Knowledge Proof System

The system uses SP1, a zero-knowledge virtual machine framework, to generate proofs that:

1. The solution follows all Sudoku rules:
   - No repeating numbers in any row
   - No repeating numbers in any column
   - No repeating numbers in any 3x3 box
   - All numbers are between 1-9

2. The solution matches the initial puzzle (all pre-filled numbers remain unchanged)

The proof can be verified without revealing the actual solution, providing privacy while ensuring correctness.

## 🛠️ Development Setup

### Prerequisites

- Rust (latest stable version)
- Cargo
- SP1 SDK

### Building the Project

```bash
# Clone the repository
git clone https://github.com/yourusername/succinct_sudoku_backend.git
cd succinct_sudoku_backend

# Build the project
cargo build --release

# Run the server
cargo run --release
```

### Build Options

- Build without including ELF binary:
  ```bash
  cargo build --release --features no_elf
  ```

## 🚢 Deployment

The project includes configuration for multiple deployment platforms:

- **Heroku/Dokku**: Uses the included `Procfile`
- **Railway**: Configuration in `railway.toml`
- **Custom Deployment**: Nixpacks configuration in `nixpacks.toml`

## 🧪 Testing

```bash
# Run tests
cargo test

# Run specific test
cargo test verify_solution
```

## 📚 Technical Implementation Details

### Job Management

The system manages proof generation jobs using:
- A shared `HashMap` protected by a `Mutex` to store job statuses
- Unique job IDs generated using UUID
- Job status tracking (Processing, Complete, Failed)

### Asynchronous Processing

Proof generation is handled asynchronously:
- Jobs are submitted and processed in the background
- Clients can check status or subscribe to WebSocket updates
- The system can handle multiple proof generation jobs concurrently

### Logging System

Comprehensive logging is implemented with:
- Request/response logging
- Job status updates
- Proof generation progress
- Error tracking

## 📄 License

This project is licensed under the terms specified in the `LICENSE` file.
