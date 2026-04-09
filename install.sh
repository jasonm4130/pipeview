#!/bin/sh
set -e

REPO="jasonm4130/pipeview"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  OS_NAME="linux" ;;
    Darwin) OS_NAME="darwin" ;;
    *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64)   ARCH_NAME="amd64" ;;
    aarch64|arm64)   ARCH_NAME="arm64" ;;
    *)               echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

BINARY_NAME="pipeview-${OS_NAME}-${ARCH_NAME}"

# Get latest release tag
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "Error: Could not determine latest release."
    echo "Try: cargo install pipeview"
    exit 1
fi

URL="https://github.com/${REPO}/releases/download/${LATEST}/${BINARY_NAME}.tar.gz"

echo "Installing pipeview ${LATEST} (${OS_NAME}/${ARCH_NAME})..."

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL "$URL" -o "${TMPDIR}/pipeview.tar.gz"
tar xzf "${TMPDIR}/pipeview.tar.gz" -C "$TMPDIR"

if [ -w "$INSTALL_DIR" ]; then
    mv "${TMPDIR}/pipeview" "${INSTALL_DIR}/pipeview"
else
    sudo mv "${TMPDIR}/pipeview" "${INSTALL_DIR}/pipeview"
fi

chmod +x "${INSTALL_DIR}/pipeview"

echo "Installed pipeview to ${INSTALL_DIR}/pipeview"
echo "Run 'pipeview --help' to get started."
