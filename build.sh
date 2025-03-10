#!/bin/bash
set -e

# Derleme ortamını hazırla
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS="-C link-arg=-s -C codegen-units=16 -C debuginfo=0 -C opt-level=z"

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