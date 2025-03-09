# Succinct Sudoku Backend

A Rust-based backend for validating Sudoku puzzles and generating Zero-Knowledge Proofs (ZKPs) using SP1.

## Features

- **Sudoku Validation**: Validate if a Sudoku board is valid
- **Solution Verification**: Verify if a solution matches the initial board
- **Zero-Knowledge Proofs**: Generate ZKPs to prove the validity of a Sudoku solution without revealing the solution
- **Asynchronous Processing**: Handle long-running proof generation in the background
- **WebSocket Support**: Track the status of proof generation in real-time

## API Endpoints

- `GET /`: Health check
- `POST /validate`: Validate a Sudoku board
- `POST /verify`: Verify a Sudoku solution against an initial board
- `POST /zkp`: Generate a simulated ZKP
- `POST /prove`: Generate a real ZKP using SP1 (asynchronous)
- `GET /proof/:job_id`: WebSocket endpoint to track proof generation status

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/alptugyaman/succinct_sudoku_backend.git
   cd succinct_sudoku_backend
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run the server:
   ```bash
   cargo run
   ```

The server will start at `http://localhost:3000`.

### Usage Examples

#### Validate a Sudoku Board

```bash
curl -X POST http://localhost:3000/validate \
  -H "Content-Type: application/json" \
  -d '{
    "board": [
      [5,3,4,6,7,8,9,1,2],
      [6,7,2,1,9,5,3,4,8],
      [1,9,8,3,4,2,5,6,7],
      [8,5,9,7,6,1,4,2,3],
      [4,2,6,8,5,3,7,9,1],
      [7,1,3,9,2,4,8,5,6],
      [9,6,1,5,3,7,2,8,4],
      [2,8,7,4,1,9,6,3,5],
      [3,4,5,2,8,6,1,7,9]
    ]
  }'
```

#### Verify a Sudoku Solution

```bash
curl -X POST http://localhost:3000/verify \
  -H "Content-Type: application/json" \
  -d '{
    "initial_board": [
      [5,3,0,0,7,0,0,0,0],
      [6,0,0,1,9,5,0,0,0],
      [0,9,8,0,0,0,0,6,0],
      [8,0,0,0,6,0,0,0,3],
      [4,0,0,8,0,3,0,0,1],
      [7,0,0,0,2,0,0,0,6],
      [0,6,0,0,0,0,2,8,0],
      [0,0,0,4,1,9,0,0,5],
      [0,0,0,0,8,0,0,7,9]
    ],
    "solution": [
      [5,3,4,6,7,8,9,1,2],
      [6,7,2,1,9,5,3,4,8],
      [1,9,8,3,4,2,5,6,7],
      [8,5,9,7,6,1,4,2,3],
      [4,2,6,8,5,3,7,9,1],
      [7,1,3,9,2,4,8,5,6],
      [9,6,1,5,3,7,2,8,4],
      [2,8,7,4,1,9,6,3,5],
      [3,4,5,2,8,6,1,7,9]
    ]
  }'
```

#### Generate a ZKP

```bash
curl -X POST http://localhost:3000/prove \
  -H "Content-Type: application/json" \
  -d '{
    "initial_board": [
      [5,3,0,0,7,0,0,0,0],
      [6,0,0,1,9,5,0,0,0],
      [0,9,8,0,0,0,0,6,0],
      [8,0,0,0,6,0,0,0,3],
      [4,0,0,8,0,3,0,0,1],
      [7,0,0,0,2,0,0,0,6],
      [0,6,0,0,0,0,2,8,0],
      [0,0,0,4,1,9,0,0,5],
      [0,0,0,0,8,0,0,7,9]
    ],
    "solution": [
      [5,3,4,6,7,8,9,1,2],
      [6,7,2,1,9,5,3,4,8],
      [1,9,8,3,4,2,5,6,7],
      [8,5,9,7,6,1,4,2,3],
      [4,2,6,8,5,3,7,9,1],
      [7,1,3,9,2,4,8,5,6],
      [9,6,1,5,3,7,2,8,4],
      [2,8,7,4,1,9,6,3,5],
      [3,4,5,2,8,6,1,7,9]
    ]
  }'
```

## Architecture

The backend is built using the following technologies:

- **Axum**: Web framework for handling HTTP requests
- **Tokio**: Asynchronous runtime
- **SP1**: Zero-Knowledge Proof system
- **WebSockets**: For real-time communication

## License

This project is licensed under the MIT License - see the LICENSE file for details. 