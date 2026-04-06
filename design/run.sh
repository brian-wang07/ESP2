#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"

cargo run --release

python3 results.py
