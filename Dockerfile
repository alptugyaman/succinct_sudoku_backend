FROM rust:1.75 as builder

WORKDIR /app
COPY . .

RUN cargo build --release --features no_elf

FROM debian:bullseye-slim

WORKDIR /app

# Gerekli kütüphaneleri kopyala
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Binary'yi builder aşamasından kopyala
COPY --from=builder /app/target/release/sudoku_backend /app/sudoku_backend

# Çalışma zamanı ortam değişkenleri
ENV RUST_LOG=info
ENV PORT=8080

# Uygulamayı çalıştır
CMD ["/app/sudoku_backend"] 