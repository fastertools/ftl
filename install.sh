#!/usr/bin/env bash
set -euo pipefail

# FTL CLI installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/fastertools/ftl-cli/main/install.sh | bash
# Options:
#   -y, --yes        Automatically answer yes to all prompts
#   -h, --help       Show help message

REPO="fastertools/ftl-cli"
BINARY_NAME="ftl"
AUTO_YES=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

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

# Show help message
show_help() {
    cat << EOF
FTL CLI Installer

Usage:
    curl -fsSL <URL> | bash
    curl -fsSL <URL> | bash -s -- [options]

Options:
    -y, --yes     Automatically answer yes to all prompts
    -h, --help    Show this help message

Examples:
    # Install latest release (interactive)
    curl -fsSL https://raw.githubusercontent.com/fastertools/ftl-cli/main/install.sh | bash

    # Install specific version
    curl -fsSL https://raw.githubusercontent.com/fastertools/ftl-cli/cli-v0.0.28/install.sh | bash

    # Automatic installation (no prompts)
    curl -fsSL https://raw.githubusercontent.com/fastertools/ftl-cli/cli-v0.0.28/install.sh | bash -s -- --yes

Note: Using the version-tagged URL (e.g., cli-v0.0.28) ensures you get the exact installer
      that was tested with that release.

EOF
    exit 0
}

# Detect OS and architecture
detect_platform() {
    local os arch

    # Detect OS
    case "$(uname -s)" in
        Linux*)     os="unknown-linux-gnu";;
        Darwin*)    os="apple-darwin";;
        *)          error "Unsupported operating system: $(uname -s)";;
    esac

    # Detect architecture
    case "$(uname -m)" in
        x86_64)     arch="x86_64";;
        amd64)      arch="x86_64";;
        aarch64)    arch="aarch64";;
        arm64)      arch="aarch64";;
        *)          error "Unsupported architecture: $(uname -m)";;
    esac

    echo "${arch}-${os}"
}

# Get the latest release version
get_latest_version() {
    local api_response
    local curl_opts="-sL"
    
    # Add authentication header if GITHUB_TOKEN is set
    if [ -n "${GITHUB_TOKEN:-}" ]; then
        curl_opts="$curl_opts -H \"Authorization: token ${GITHUB_TOKEN}\""
    fi
    
    api_response=$(eval "curl $curl_opts \"https://api.github.com/repos/${REPO}/releases/latest\"")
    
    # Debug: Show first 200 chars of response if it looks like an error
    if echo "$api_response" | grep -q '"message"'; then
        info "API Response: $(echo "$api_response" | head -c 200)..."
    fi
    
    # Check if we got a valid response
    if [ -z "$api_response" ] || echo "$api_response" | grep -q "Not Found"; then
        return 1
    fi
    
    # Extract version from tag_name (format: cli-vX.Y.Z)
    echo "$api_response" | grep -o '"tag_name":\s*"cli-v[^"]*"' | sed -E 's/.*"cli-v([^"]+)".*/\1/'
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check dependencies
check_dependencies() {
    local missing_deps=()
    
    # Check for Rust
    if ! command_exists "cargo"; then
        missing_deps+=("rust")
    fi
    
    # Check for Spin
    if ! command_exists "spin"; then
        missing_deps+=("spin")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo ""
        info "Missing required dependencies: ${missing_deps[*]}"
        echo ""
        
        # Provide installation instructions
        for dep in "${missing_deps[@]}"; do
            case "$dep" in
                rust)
                    echo "To install Rust:"
                    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                    echo "  source \$HOME/.cargo/env"
                    echo ""
                    ;;
                spin)
                    echo "To install Spin:"
                    echo "  curl -fsSL https://developer.fermyon.com/downloads/fwf_install.sh | bash"
                    echo "  sudo mv ./spin /usr/local/bin/spin"
                    echo ""
                    ;;
            esac
        done
        
        if [ "$AUTO_YES" = true ]; then
            info "Continuing installation (--yes flag provided)"
        else
            read -p "Would you like to continue anyway? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                error "Installation cancelled. Please install missing dependencies first."
            fi
        fi
        echo ""
        info "⚠️  Warning: FTL requires Rust and Spin to function properly"
        echo ""
    else
        success "✓ All dependencies found"
    fi
}

# Download and install
main() {
    info "Installing FTL CLI..."

    # Check dependencies first
    check_dependencies

    # Detect platform
    local platform=$(detect_platform)
    info "Detected platform: ${platform}"

    # Get latest version
    local version=$(get_latest_version)
    if [ -z "$version" ]; then
        error "Failed to determine latest version. Please check:
  - Is the repository public or are you authenticated?
  - Are there any releases published?
  - API URL: https://api.github.com/repos/${REPO}/releases/latest"
    fi
    info "Latest version: v${version}"

    # Construct download URL
    local asset_name="${BINARY_NAME}-${platform}"
    local download_url="https://github.com/${REPO}/releases/download/cli-v${version}/${asset_name}"
    
    info "Downloading from: ${download_url}"

    # Download binary
    if ! curl -fsSL "${download_url}" -o "${BINARY_NAME}"; then
        error "Failed to download ${BINARY_NAME} binary"
    fi

    # Make executable
    chmod +x "${BINARY_NAME}"

    success "✓ FTL CLI v${version} downloaded successfully!"
    echo ""
    echo "To install system-wide, run:"
    echo "  sudo mv ./${BINARY_NAME} /usr/local/bin/${BINARY_NAME}"
    echo ""
    echo "Or add to your PATH:"
    echo "  mkdir -p ~/.local/bin"
    echo "  mv ./${BINARY_NAME} ~/.local/bin/${BINARY_NAME}"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Then verify installation with:"
    echo "  ${BINARY_NAME} --version"
    
    # Final dependency reminder
    if ! command_exists "cargo" || ! command_exists "spin"; then
        echo ""
        info "Remember: FTL requires Rust and Spin to be installed for full functionality"
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -y|--yes)
            AUTO_YES=true
            shift
            ;;
        -h|--help)
            show_help
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

main