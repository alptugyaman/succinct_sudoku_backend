FROM rust:1.76-slim

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire project
COPY . .

# Create the directory for the ELF file
RUN mkdir -p target/elf-compilation/riscv32im-succinct-zkvm-elf/release/

# Build the SP1 prover
RUN cd sp1_prover && cargo build --release

# Copy the compiled SP1 prover to the expected location
RUN cp sp1_prover/target/release/sp1_prover target/elf-compilation/riscv32im-succinct-zkvm-elf/release/

# Build the application
RUN cargo build --release

# Create assets directory if needed
RUN mkdir -p /app/assets

# Expose the port the app will run on
ENV PORT=3000
EXPOSE 3000

# Command to run the application
CMD ["./target/release/sudoku_backend"] 