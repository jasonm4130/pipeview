#!/bin/sh
set -e

REPO="jasonm4130/pipespy"
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

BINARY_NAME="pipespy-${OS_NAME}-${ARCH_NAME}"

# Get latest release tag
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "Error: Could not determine latest release."
    echo "Try: cargo install pipespy"
    exit 1
fi

URL="https://github.com/${REPO}/releases/download/${LATEST}/${BINARY_NAME}.tar.gz"

echo "Installing pipespy ${LATEST} (${OS_NAME}/${ARCH_NAME})..."

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL "$URL" -o "${TMPDIR}/pipespy.tar.gz"
tar xzf "${TMPDIR}/pipespy.tar.gz" -C "$TMPDIR"

if [ -w "$INSTALL_DIR" ]; then
    mv "${TMPDIR}/pipespy" "${INSTALL_DIR}/pipespy"
else
    sudo mv "${TMPDIR}/pipespy" "${INSTALL_DIR}/pipespy"
fi

chmod +x "${INSTALL_DIR}/pipespy"

echo "Installed pipespy to ${INSTALL_DIR}/pipespy"
echo "Run 'pipespy --help' to get started."
