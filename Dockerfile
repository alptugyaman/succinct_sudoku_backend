FROM rust:1.76-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty project
WORKDIR /app
COPY . .

# Install SP1 CLI tools directly from GitHub
RUN git clone https://github.com/succinctlabs/sp1.git /tmp/sp1 && \
    cd /tmp/sp1 && \
    cargo install --path sp1-cli

# Add cargo bin to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Verify SP1 installation
RUN which cargo-prove || echo "cargo-prove not found in PATH"

# Try multiple approaches to build the SP1 prover
RUN cd sp1_prover && \
    if which cargo-prove > /dev/null; then \
    echo "Using cargo-prove to build" && \
    cargo prove build; \
    else \
    echo "Attempting manual build" && \
    # First try: Use rustup to add the target
    rustup target add riscv32im-unknown-none-elf || true && \
    # Build with the target
    cargo build --release --target riscv32im-unknown-none-elf || \
    # Second try: Build without specific target
    cargo build --release; \
    fi

# Ensure the target directory exists
RUN mkdir -p /app/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/

# Try to find and copy the built SP1 prover to the expected location
RUN find /app -name "sp1_prover" -type f -executable | \
    while read file; do \
    echo "Found executable: $file"; \
    cp "$file" /app/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/sp1_prover || true; \
    done

# If we still don't have the prover, create a dummy one (this is a fallback)
RUN if [ ! -f /app/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/sp1_prover ]; then \
    echo "Creating dummy prover file"; \
    echo "#!/bin/sh" > /app/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/sp1_prover; \
    chmod +x /app/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/sp1_prover; \
    fi

# Build the main application with the dummy_prover feature
RUN cargo build --release --features dummy_prover

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