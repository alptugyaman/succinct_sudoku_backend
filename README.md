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
- `POST /api/prove`: Generate a ZKP for a Sudoku solution (asynchronous)
- `GET /api/proof/:job_id`: WebSocket endpoint to track proof generation status
- `GET /api/logs/:job_id`: WebSocket endpoint to track logs for a specific job

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

## API Documentation

### Generate a ZKP

**Endpoint:** `POST /api/prove`

**Description:** Initiates the generation of a Zero-Knowledge Proof for a Sudoku solution. This is an asynchronous operation that returns a job ID immediately.

**Request Body:**
```json
{
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
}
```

**Response:**
```json
{
  "job_id": "d3ad9fe8-290f-466b-9c36-64a5cce26fec",
  "status": "processing",
  "result": null,
  "error": null
}
```

**Example:**
```bash
curl -X POST http://localhost:3000/api/prove \
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

### Track Proof Generation Status

**Endpoint:** `GET /api/proof/:job_id`

**Description:** WebSocket endpoint to track the status of a proof generation job in real-time.

**Parameters:**
- `job_id`: The ID of the job returned from the `/api/prove` endpoint

**WebSocket Messages:**

1. **Processing:**
```json
{
  "job_id": "d3ad9fe8-290f-466b-9c36-64a5cce26fec",
  "status": "processing",
  "result": null,
  "error": null
}
```

2. **Complete:**
```json
{
  "job_id": "d3ad9fe8-290f-466b-9c36-64a5cce26fec",
  "status": "complete",
  "result": {
    "public_values": "true",
    "proof": "proof-d3ad9fe8-290f-466b-9c36-64a5cce26fec.proof"
  },
  "error": null
}
```

3. **Failed:**
```json
{
  "job_id": "d3ad9fe8-290f-466b-9c36-64a5cce26fec",
  "status": "failed",
  "result": null,
  "error": "Invalid solution"
}
```

**Example (JavaScript):**
```javascript
const socket = new WebSocket(`ws://localhost:3000/api/proof/${jobId}`);

socket.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Proof status:', data.status);
  
  if (data.status === 'complete') {
    console.log('Proof generated:', data.result);
    socket.close();
  } else if (data.status === 'failed') {
    console.error('Proof generation failed:', data.error);
    socket.close();
  }
};
```

### Track Job Logs

**Endpoint:** `GET /api/logs/:job_id`

**Description:** WebSocket endpoint to track logs for a specific job in real-time.

**Parameters:**
- `job_id`: The ID of the job returned from the `/api/prove` endpoint

**WebSocket Messages:**
The server will send log messages as they are generated. Each message is a plain text string.

**Example (JavaScript):**
```javascript
const logsSocket = new WebSocket(`ws://localhost:3000/api/logs/${jobId}`);

logsSocket.onmessage = (event) => {
  console.log('Log:', event.data);
};
```

## Frontend Integration Guide

### Step 1: Submit a Sudoku Puzzle for Proof Generation

```javascript
async function generateProof(initialBoard, solution) {
  const response = await fetch('http://localhost:3000/api/prove', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      initial_board: initialBoard,
      solution: solution
    }),
  });
  
  const data = await response.json();
  return data.job_id;
}
```

### Step 2: Track Proof Generation Status

```javascript
function trackProofStatus(jobId, callbacks) {
  const { onProcessing, onComplete, onFailed } = callbacks;
  
  const socket = new WebSocket(`ws://localhost:3000/api/proof/${jobId}`);
  
  socket.onopen = () => {
    console.log('WebSocket connection established');
  };
  
  socket.onmessage = (event) => {
    const data = JSON.parse(event.data);
    
    switch (data.status) {
      case 'processing':
        onProcessing && onProcessing();
        break;
      case 'complete':
        onComplete && onComplete(data.result);
        socket.close();
        break;
      case 'failed':
        onFailed && onFailed(data.error);
        socket.close();
        break;
      default:
        console.warn('Unknown status:', data.status);
    }
  };
  
  socket.onerror = (error) => {
    console.error('WebSocket error:', error);
    onFailed && onFailed('WebSocket connection error');
  };
  
  socket.onclose = () => {
    console.log('WebSocket connection closed');
  };
  
  // Return a function to close the socket
  return () => socket.close();
}
```

### Step 3: Track Job Logs

```javascript
function trackJobLogs(jobId, onLog) {
  const logsSocket = new WebSocket(`ws://localhost:3000/api/logs/${jobId}`);
  
  logsSocket.onopen = () => {
    console.log('Logs WebSocket connection established');
  };
  
  logsSocket.onmessage = (event) => {
    onLog && onLog(event.data);
  };
  
  logsSocket.onerror = (error) => {
    console.error('Logs WebSocket error:', error);
  };
  
  logsSocket.onclose = () => {
    console.log('Logs WebSocket connection closed');
  };
  
  // Return a function to close the socket
  return () => logsSocket.close();
}
```

### Step 4: Complete Example

```javascript
async function verifySudokuSolution(initialBoard, solution) {
  try {
    // Step 1: Submit the puzzle for proof generation
    const jobId = await generateProof(initialBoard, solution);
    console.log('Proof generation started with job ID:', jobId);
    
    // Step 2: Set up UI for tracking status
    const statusElement = document.getElementById('status');
    const logsElement = document.getElementById('logs');
    
    statusElement.textContent = 'Processing...';
    
    // Step 3: Track logs
    const closeLogsSocket = trackJobLogs(jobId, (log) => {
      const logLine = document.createElement('div');
      logLine.textContent = log;
      logsElement.appendChild(logLine);
      logsElement.scrollTop = logsElement.scrollHeight;
    });
    
    // Step 4: Track proof status
    trackProofStatus(jobId, {
      onProcessing: () => {
        statusElement.textContent = 'Processing proof...';
      },
      onComplete: (result) => {
        statusElement.textContent = 'Proof generation complete!';
        console.log('Proof result:', result);
        
        // Display the proof result
        const resultElement = document.getElementById('result');
        resultElement.textContent = `Proof: ${result.proof}`;
        
        // Close logs socket when complete
        closeLogsSocket();
      },
      onFailed: (error) => {
        statusElement.textContent = `Error: ${error}`;
        console.error('Proof generation failed:', error);
        
        // Close logs socket on failure
        closeLogsSocket();
      }
    });
  } catch (error) {
    console.error('Error:', error);
  }
}
```

## Architecture

The backend is built using the following technologies:

- **Axum**: Web framework for handling HTTP requests
- **Tokio**: Asynchronous runtime
- **SP1**: Zero-Knowledge Proof system
- **WebSockets**: For real-time communication

### Proof Generation Flow

1. Client submits a Sudoku puzzle and solution to `/api/prove`
2. Server validates the input and creates a new job with a unique ID
3. Server returns the job ID immediately and starts proof generation in the background
4. Client connects to WebSocket endpoints to track status and logs
5. Server generates the proof and stores it in the `assets/` directory
6. Server updates the job status to "complete" or "failed"
7. Client receives the final status via WebSocket

### Logging System

The backend implements a comprehensive logging system:

1. **HTTP Request Logging**: All HTTP requests are logged with method, path, status code, and duration
2. **Request Body Logging**: Request bodies are logged for debugging purposes
3. **Job-Specific Logging**: Each proof generation job has its own log stream
4. **WebSocket Log Streaming**: Logs are streamed to clients in real-time via WebSocket

## License

This project is licensed under the MIT License - see the LICENSE file for details. 