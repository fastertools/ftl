#!/usr/bin/env bash

# FTL CLI Installation Script
# Detects platform and architecture, downloads the appropriate binary

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Configuration
REPO="fastertools/ftl-cli"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="ftl"

# Helper functions
error() {
    echo -e "${RED}Error: $1${NC}" >&2
    exit 1
}

success() {
    echo -e "${GREEN}$1${NC}"
}

info() {
    echo -e "${YELLOW}$1${NC}"
}

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    
    case "$OS" in
        Linux*)
            PLATFORM="linux"
            ;;
        Darwin*)
            PLATFORM="darwin"
            ;;
        *)
            error "Unsupported operating system: $OS"
            ;;
    esac
    
    case "$ARCH" in
        x86_64|amd64)
            ARCH="amd64"
            ;;
        arm64|aarch64)
            ARCH="arm64"
            ;;
        *)
            error "Unsupported architecture: $ARCH"
            ;;
    esac
    
    echo "${PLATFORM}-${ARCH}"
}

# Get the latest release version
get_latest_version() {
    local url="https://api.github.com/repos/${REPO}/releases/latest"
    
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

# Download and install the binary
install_ftl() {
    local version="${1:-$(get_latest_version)}"
    local platform="$(detect_platform)"
    
    if [ -z "$version" ]; then
        error "Could not determine the latest version"
    fi
    
    info "Installing FTL CLI ${version} for ${platform}..."
    
    local archive_name="ftl-${version}-${platform}.tar.gz"
    local download_url="https://github.com/${REPO}/releases/download/${version}/${archive_name}"
    
    # Create temporary directory
    local tmp_dir="$(mktemp -d)"
    trap "rm -rf $tmp_dir" EXIT
    
    cd "$tmp_dir"
    
    # Download the archive
    info "Downloading from ${download_url}..."
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$archive_name" "$download_url" || error "Failed to download FTL CLI"
    else
        wget -q -O "$archive_name" "$download_url" || error "Failed to download FTL CLI"
    fi
    
    # Extract the archive
    info "Extracting..."
    tar xzf "$archive_name" || error "Failed to extract archive"
    
    # Find the binary
    local binary_path="ftl"
    if [ ! -f "$binary_path" ]; then
        error "Binary not found in archive"
    fi
    
    # Check if we need sudo for installation
    local use_sudo=""
    if [ "$INSTALL_DIR" = "/usr/local/bin" ] && [ "$EUID" -ne 0 ]; then
        if command -v sudo >/dev/null 2>&1; then
            use_sudo="sudo"
            info "Installing to $INSTALL_DIR (requires sudo)..."
        else
            error "Installation to $INSTALL_DIR requires root privileges"
        fi
    fi
    
    # Install the binary
    chmod +x "$binary_path"
    $use_sudo mkdir -p "$INSTALL_DIR"
    $use_sudo mv "$binary_path" "$INSTALL_DIR/$BINARY_NAME"
    
    # Verify installation
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        success "FTL CLI installed successfully!"
        info "Version: $("$BINARY_NAME" --version)"
    else
        if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
            info "FTL CLI installed to $INSTALL_DIR/$BINARY_NAME"
            info "Make sure $INSTALL_DIR is in your PATH"
        else
            error "Installation failed"
        fi
    fi
}

# Main execution
main() {
    echo "FTL CLI Installer"
    echo "================="
    echo
    
    # Parse arguments
    local version=""
    while [ $# -gt 0 ]; do
        case "$1" in
            --version|-v)
                version="$2"
                shift 2
                ;;
            --install-dir|-d)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  -v, --version VERSION    Install a specific version"
                echo "  -d, --install-dir DIR    Installation directory (default: /usr/local/bin)"
                echo "  -h, --help              Show this help message"
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                ;;
        esac
    done
    
    install_ftl "$version"
}

main "$@"