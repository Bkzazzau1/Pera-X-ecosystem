#!/usr/bin/env bash
set -euo pipefail

sudo apt-get update
sudo apt-get install -y \
  build-essential \
  ca-certificates \
  clang \
  cmake \
  curl \
  git \
  libssl-dev \
  pkg-config \
  protobuf-compiler \
  redis-server \
  unzip \
  wget \
  zip

if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fi

# shellcheck source=/dev/null
source "$HOME/.cargo/env"

rustup toolchain install stable
rustup default stable
rustup component add rustfmt clippy

if ! command -v nvm >/dev/null 2>&1; then
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
fi

export NVM_DIR="$HOME/.nvm"
# shellcheck source=/dev/null
[ -s "$NVM_DIR/nvm.sh" ] && source "$NVM_DIR/nvm.sh"
nvm install --lts
nvm use --lts
corepack enable

if ! command -v avm >/dev/null 2>&1; then
  cargo install --git https://github.com/coral-xyz/anchor avm --locked
fi

avm install latest
avm use latest

sudo service redis-server start

if ! grep -q "PERAX DEV DEFAULTS" "$HOME/.bashrc"; then
  cat >> "$HOME/.bashrc" <<'EOF'

# PERAX DEV DEFAULTS
export PATH="$HOME/.cargo/bin:$PATH"
export RUST_BACKTRACE=1
alias ll='ls -lah'
alias redis-start='sudo service redis-server start'
alias perax='cd /mnt/c/PROJECTS/"smartcontract PEX"/perax-ecosystem'
EOF
fi

echo ""
echo "Perax WSL dev environment is ready."
echo "Versions:"
rustc --version
cargo --version
node --version
npm --version
redis-server --version
anchor --version || true

