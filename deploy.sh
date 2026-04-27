#!/bin/bash
# Rouman OS - Integrated Deployment Script
set -e

echo "🚀 Starting Rouman OS Production Build..."

# 1. Build eBPF (Kernel Layer)
echo "🛡️ Building eBPF Security Layer (Release Mode)..."
cargo +nightly build --release --target bpfel-unknown-none -Z build-std=core -p rouman-ebpf

# 2. Build Frontend
echo "📦 Checking dependencies and Building UI..."
cd ui
if [ ! -d "node_modules" ]; then
    npm install
fi
npm run build
cd ..

# 3. Start Project
echo "🧹 Cleaning up existing ports..."
fuser -k 8000/tcp 3000/tcp 2>/dev/null || true
sleep 1

echo "🚀 Starting Rouman OS Gateway (Integrated Mode - RELEASE)..."
echo "👉 Access at: http://localhost:8000"
# Run daemon in release mode for maximum performance and stability
cargo run --release --bin rouman-daemon
