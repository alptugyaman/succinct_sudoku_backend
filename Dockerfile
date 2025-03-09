FROM rust:1.76-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install SP1 CLI tools
RUN curl -sSf https://raw.githubusercontent.com/succinctlabs/sp1/main/sp1up/install.sh | sh

# Add SP1 to PATH
ENV PATH="/root/.sp1/bin:${PATH}"

# Create a new empty project
WORKDIR /app
COPY . .

# Build the SP1 prover
RUN cd sp1_prover && cargo prove build

# Build the main application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create necessary directories
RUN mkdir -p /app/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/

# Copy the built binaries and necessary files
COPY --from=builder /app/target/release/sudoku_backend /app/
COPY --from=builder /app/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/sp1_prover /app/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/

# Set the PORT environment variable for Railway
ENV PORT=3000

# Expose the port
EXPOSE ${PORT}

# Run the application
CMD ["./sudoku_backend"] 