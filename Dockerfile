FROM rust:1.76-slim as builder

WORKDIR /app

# Bağımlılıkları önce kopyala ve derle (önbellek için)
COPY Cargo.toml Cargo.lock ./
COPY sp1_prover/Cargo.toml ./sp1_prover/
RUN mkdir -p src sp1_prover/src && \
    touch src/lib.rs && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > sp1_prover/src/main.rs

# Bağımlılıkları derle
RUN cargo build --release

# Gerçek kaynak kodunu kopyala
COPY . .

# Yeniden derle
RUN cargo build --release

# Çalışma zamanı imajı
FROM debian:bullseye-slim

WORKDIR /app

# Gerekli çalışma zamanı bağımlılıklarını kur
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Derlenen binary'yi kopyala
COPY --from=builder /app/target/release/sudoku_backend /app/
COPY --from=builder /app/target/elf-compilation /app/target/elf-compilation

# Assets dizinini oluştur
RUN mkdir -p /app/assets

# Ortam değişkenlerini ayarla
ENV RUST_LOG=info

# Portu aç
EXPOSE 3000

# Uygulamayı çalıştır
CMD ["./sudoku_backend"] 