#!/bin/bash
# installs what SysQL needs (bpftrace, a C linker, Rust), builds it, and puts `sysql` on your PATH
set -e
cd "$(dirname "$0")"

if [[ $(uname) != Linux ]]; then
    echo "SysQL needs Linux. On a Mac, start a Linux VM first (see the README)."
    exit 1
fi

if ! command -v bpftrace > /dev/null || ! command -v cc > /dev/null; then
    sudo apt-get update
    sudo apt-get install -y bpftrace build-essential
fi

if ! command -v cargo > /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

cargo build --release --target-dir target
sudo install -m 755 target/release/sysql /usr/local/bin/sysql
echo
echo "SysQL is installed. Type: sysql"
