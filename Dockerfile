FROM rust:1.76-slim-bullseye as builder

WORKDIR /app

# Gerekli paketleri kur
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev build-essential git && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Cargo yapılandırmasını kopyala
COPY .cargo /app/.cargo

# Cargo.toml ve Cargo.lock dosyalarını kopyala
COPY Cargo.toml Cargo.lock ./
COPY sp1_prover/Cargo.toml ./sp1_prover/

# Sahte bir main.rs oluştur ve bağımlılıkları önceden derle
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p sp1_prover/src && \
    echo "fn main() {}" > sp1_prover/src/main.rs && \
    RUSTFLAGS="-C link-arg=-s -C codegen-units=16 -C debuginfo=0 -C opt-level=z" \
    cargo build --release --features no_elf && \
    rm -rf src sp1_prover/src

# Gerçek kaynak kodunu kopyala
COPY src ./src
COPY sp1_prover/src ./sp1_prover/src

# Projeyi derle
RUN RUSTFLAGS="-C link-arg=-s -C codegen-units=16 -C debuginfo=0 -C opt-level=z" \
    cargo build --release --features no_elf

# Çalışma zamanı aşaması
FROM debian:bullseye-slim

WORKDIR /app

# Gerekli çalışma zamanı bağımlılıklarını kur
RUN apt-get update && \
    apt-get install -y ca-certificates libssl-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Derlenen binary'yi kopyala
COPY --from=builder /app/target/release/sudoku_backend /app/sudoku_backend

# Ortam değişkenlerini ayarla
ENV RUST_LOG=info
ENV PORT=8080

# Uygulamayı çalıştır
CMD ["/app/sudoku_backend"] 