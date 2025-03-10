#!/bin/bash
set -e

# Rust ve Cargo'nun kurulu olduğundan emin ol
if ! command -v rustc &> /dev/null; then
    echo "Rust kurulu değil, kurulum yapılıyor..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# PATH'e Cargo'yu ekle
export PATH="$HOME/.cargo/bin:$PATH"

# Derleme ortamını hazırla
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS="-C link-arg=-s -C codegen-units=16 -C debuginfo=0 -C opt-level=z"

# Cargo'nun mevcut olduğunu kontrol et
which cargo || { echo "Cargo bulunamadı!"; exit 1; }

# Önce bağımlılıkları derle
echo "Bağımlılıklar derleniyor..."
cargo fetch

# Bağımlılıkları önceden derle
echo "Bağımlılıklar önceden derleniyor..."
cargo build --release --features no_elf --lib

# Ana projeyi derle
echo "Ana proje derleniyor..."
cargo build --release --features no_elf

echo "Derleme tamamlandı!" 