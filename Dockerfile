FROM rust:1.76-slim as builder

WORKDIR /app

# Gerekli paketleri yükle
RUN apt-get update && \
    apt-get install -y pkg-config build-essential libssl-dev curl git && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Cargo.toml ve Cargo.lock dosyalarını kopyala
COPY Cargo.toml Cargo.lock ./
COPY sp1_prover/Cargo.toml ./sp1_prover/

# Sahte bir main.rs oluştur ve bağımlılıkları önceden derle
RUN mkdir -p sp1_prover/src && \
    echo "fn main() {}" > sp1_prover/src/main.rs && \
    cargo build --release && \
    rm -rf sp1_prover/src

# Gerçek kaynak kodunu kopyala
COPY sp1_prover/src ./sp1_prover/src

# Projeyi derle
RUN cargo build --release

# Çalışma zamanı aşaması
FROM debian:bullseye-slim

WORKDIR /app

# Gerekli çalışma zamanı bağımlılıklarını yükle
RUN apt-get update && \
    apt-get install -y libssl-dev ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Derlenmiş binary'yi kopyala
COPY --from=builder /app/target/release/sp1_prover /app/sp1_prover

# Çalıştırma komutu
CMD ["/app/sp1_prover"] 