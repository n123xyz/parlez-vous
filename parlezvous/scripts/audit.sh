#!/bin/bash
set -e

echo "Running Supply Chain Audit..."

# Check for cargo-audit
if ! command -v cargo-audit &> /dev/null
then
    echo "cargo-audit could not be found. Installing..."
    cargo install cargo-audit
fi

# Check for cargo-deny
if ! command -v cargo-deny &> /dev/null
then
    echo "cargo-deny could not be found. Installing..."
    cargo install cargo-deny
fi

cd src-tauri

echo "Running cargo audit (Vulnerability check)..."
cargo audit

echo "Running cargo deny check (Bans, Licenses, Advisories, Sources)..."
if [ ! -f "deny.toml" ]; then
    echo "Initializing cargo deny config..."
    cargo deny init
fi
cargo deny check

echo "Audit completed successfully."
