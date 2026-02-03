#!/usr/bin/env bash
set -e

REPO="https://github.com/XMA-Faez/yt-transcriber.git"
INSTALL_DIR="${YT_TRANSCRIBER_DIR:-$HOME/.yt-transcriber}"

echo "Installing yt-transcriber..."

if ! command -v git &> /dev/null; then
  echo "Error: git is required but not installed."
  exit 1
fi

if ! command -v cargo &> /dev/null; then
  echo "Rust not found. Installing via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
fi

if [ -d "$INSTALL_DIR" ]; then
  echo "Updating existing installation..."
  cd "$INSTALL_DIR"
  git pull --ff-only
else
  echo "Cloning repository..."
  git clone "$REPO" "$INSTALL_DIR"
  cd "$INSTALL_DIR"
fi

echo "Building release binary..."
cargo build --release

echo "Installing binary..."
mkdir -p "$HOME/.local/bin"
cp target/release/yt-transcriber "$HOME/.local/bin/"

if ! command -v yt-dlp &> /dev/null; then
  echo "Installing yt-dlp..."
  if command -v pip &> /dev/null; then
    pip install --user yt-dlp
  elif command -v pipx &> /dev/null; then
    pipx install yt-dlp
  elif command -v brew &> /dev/null; then
    brew install yt-dlp
  else
    echo "Warning: Could not install yt-dlp automatically."
    echo "Please install it manually: pip install yt-dlp"
  fi
fi

echo ""
echo "yt-transcriber installed successfully!"
echo ""
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
  echo "Add ~/.local/bin to your PATH by adding this to your shell config:"
  echo '  export PATH="$HOME/.local/bin:$PATH"'
  echo ""
fi
echo "Run 'yt-transcriber --help' to get started."
