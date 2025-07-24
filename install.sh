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

# Execute command with optional confirmation
# Usage: confirm_exec "description" command args...
confirm_exec() {
    local description="$1"
    shift
    
    if [ "$AUTO_YES" = false ]; then
        echo ""
        info "About to: $description"
        echo "Command: $*"
        read -p "Continue? [Y/n] " -n 1 -r </dev/tty
        echo
        if [[ $REPLY =~ ^[Nn]$ ]]; then
            error "Operation cancelled by user"
        fi
    fi
    
    # Execute the command
    "$@"
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

    # For private repos (requires gh CLI to be authenticated)
    gh auth login  # If not already authenticated
    curl -fsSL https://$(gh auth token)@raw.githubusercontent.com/fastertools/ftl-cli/main/install.sh | bash

Note: This installer requires GitHub CLI (gh) to be installed and authenticated.
      Install gh from: https://cli.github.com

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
    # Get the latest release tag that starts with cli-v
    local latest_tag=$(confirm_exec "List GitHub releases to find latest version" gh release list --repo "${REPO}" --limit 10 | grep -E "^cli-v[0-9]" | head -1 | awk '{print $1}')
    if [ -n "$latest_tag" ]; then
        # Extract version from tag (format: cli-vX.Y.Z)
        echo "$latest_tag" | sed 's/^cli-v//'
    else
        return 1
    fi
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check dependencies
check_dependencies() {
    info "Checking dependencies..."
    echo ""
    
    local missing_deps=()
    
    # Check for gh CLI
    echo -n "  Checking for GitHub CLI (gh)... "
    if ! command_exists "gh"; then
        echo "❌ not found"
        missing_deps+=("gh")
    else
        echo "✓ found"
        
        # Check gh authentication
        echo -n "  Checking GitHub CLI authentication... "
        if gh auth status >/dev/null 2>&1; then
            echo "✓ authenticated"
        else
            echo "❌ not authenticated"
            error "GitHub CLI is not authenticated. Please run: gh auth login"
        fi
    fi
    
    # Check for Rust
    echo -n "  Checking for Rust (cargo)... "
    if ! command_exists "cargo"; then
        echo "❌ not found"
        missing_deps+=("rust")
    else
        echo "✓ found"
        
        # Check for wasm32-wasip1 target
        echo -n "  Checking for wasm32-wasip1 target... "
        if rustup target list --installed | grep -q "wasm32-wasip1"; then
            echo "✓ installed"
        else
            echo "❌ not installed"
            missing_deps+=("wasm-target")
        fi
    fi
    
    # Check for Spin
    echo -n "  Checking for Spin... "
    if ! command_exists "spin"; then
        echo "❌ not found"
        missing_deps+=("spin")
    else
        echo "✓ found"
    fi
    
    echo ""
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo ""
        info "Missing required dependencies: ${missing_deps[*]}"
        echo ""
        
        # Provide installation instructions
        for dep in "${missing_deps[@]}"; do
            case "$dep" in
                gh)
                    echo "To install GitHub CLI:"
                    echo "  Visit: https://cli.github.com"
                    echo "  After installing, run: gh auth login"
                    echo ""
                    ;;
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
                wasm-target)
                    echo "To install wasm32-wasip1 target:"
                    echo "  rustup target add wasm32-wasip1"
                    echo ""
                    ;;
            esac
        done
        
        if [ "$AUTO_YES" = true ]; then
            info "Continuing installation (--yes flag provided)"
        else
            read -p "Would you like to continue anyway? (y/N) " -n 1 -r </dev/tty
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                error "Installation cancelled. Please install missing dependencies first."
            fi
        fi
        echo ""
        info "⚠️  Warning: FTL requires gh CLI, Rust (with wasm32-wasip1 target), and Spin to function properly"
        echo ""
    else
        success "✓ All dependencies found"
    fi
}

# Download and install
main() {
    info "Installing FTL CLI..."
    
    # Show what we'll do (unless in auto mode)
    if [ "$AUTO_YES" = false ]; then
        echo ""
        info "This script will:"
        echo "  1. Check for required dependencies (gh CLI, Rust with wasm target, Spin)"
        echo "  2. Verify GitHub CLI authentication"
        echo "  3. Find the latest FTL release"
        echo "  4. Download the FTL binary"
        echo "  5. Install it to ~/.local/bin or /usr/local/bin"
        echo "  6. Set up FTL templates"
        echo ""
        read -p "Continue? [Y/n] " -n 1 -r </dev/tty
        echo
        if [[ $REPLY =~ ^[Nn]$ ]]; then
            error "Installation cancelled"
        fi
    fi

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

    # Construct asset name
    local asset_name="${BINARY_NAME}-${platform}"
    
    info "Downloading ${asset_name} using gh CLI..."
    
    # Download using gh release download
    if ! confirm_exec "Download ftl binary from GitHub release" \
        gh release download "cli-v${version}" \
        --repo "${REPO}" \
        --pattern "${asset_name}" \
        --output "${BINARY_NAME}"; then
        error "Failed to download ${BINARY_NAME} binary. Make sure you have gh CLI authenticated: gh auth login"
    fi

    # Make executable
    chmod +x "${BINARY_NAME}"

    success "✓ FTL CLI v${version} downloaded successfully!"
    echo ""
    
    # Ask user where to install
    if [ "$AUTO_YES" = true ]; then
        # Default to ~/.local/bin in auto mode
        INSTALL_CHOICE="1"
    else
        info "Where would you like to install ftl?"
        echo ""
        echo "  1) ~/.local/bin (user directory) - Recommended"
        echo "  2) /usr/local/bin (requires sudo)"
        echo "  3) Skip installation (manual setup)"
        echo ""
        read -p "Select an option [1-3]: " -n 1 -r INSTALL_CHOICE </dev/tty
        echo
    fi
    
    case "$INSTALL_CHOICE" in
        1|"")
            # Install to ~/.local/bin
            info "Installing to ~/.local/bin..."
            mkdir -p ~/.local/bin
            mv ./${BINARY_NAME} ~/.local/bin/${BINARY_NAME}
            INSTALL_PATH="$HOME/.local/bin/${BINARY_NAME}"
            success "✓ Installed to ~/.local/bin/${BINARY_NAME}"
            
            # Check if ~/.local/bin is in PATH
            if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
                echo ""
                info "⚠️  ~/.local/bin is not in your PATH"
                echo ""
                echo "Add this line to your shell configuration file:"
                echo ""
                # Detect shell
                if [[ "$SHELL" == *"zsh"* ]]; then
                    echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
                    echo ""
                    echo "Then reload your shell configuration:"
                    echo "  source ~/.zshrc"
                elif [[ "$SHELL" == *"bash"* ]]; then
                    echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
                    echo ""
                    echo "Then reload your shell configuration:"
                    echo "  source ~/.bashrc"
                else
                    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
                    echo ""
                    echo "Add the above line to your shell's configuration file"
                fi
            fi
            ;;
        2)
            # Install to /usr/local/bin
            info "Installing to /usr/local/bin..."
            if sudo mv ./${BINARY_NAME} /usr/local/bin/${BINARY_NAME}; then
                INSTALL_PATH="/usr/local/bin/${BINARY_NAME}"
                success "✓ Installed to /usr/local/bin/${BINARY_NAME}"
            else
                error "Failed to install to /usr/local/bin. Please run manually: sudo mv ./${BINARY_NAME} /usr/local/bin/${BINARY_NAME}"
            fi
            ;;
        3)
            # Skip installation
            info "Installation skipped. To install manually:"
            echo ""
            echo "  # Move the binary to your preferred location:"
            echo "  mv ./${BINARY_NAME} /path/to/destination/"
            echo ""
            echo "  # Make it executable (if needed):"
            echo "  chmod +x /path/to/destination/${BINARY_NAME}"
            echo ""
            echo "  # Add the directory to your PATH if needed"
            echo ""
            info "To continue with template setup:"
            echo "  ./${BINARY_NAME} setup templates"
            echo ""
            # Exit early since we're not installing
            exit 0
            ;;
        *)
            error "Invalid option selected"
            ;;
    esac
    
    # Verify installation
    if command_exists "${BINARY_NAME}"; then
        echo ""
        success "✓ FTL CLI is ready to use!"
        
        # Setup templates automatically
        echo ""
        info "Setting up FTL templates..."
        if confirm_exec "Download and install FTL templates from GitHub" ${BINARY_NAME} setup templates; then
            success "✓ Templates installed successfully!"
        else
            info "⚠️  Template setup failed. You can run it manually later with: ftl setup templates"
        fi
    else
        echo ""
        info "Verify installation with:"
        echo "  ${BINARY_NAME} --version"
    fi
    
    # Final dependency reminder
    if ! command_exists "gh" || ! command_exists "cargo" || ! command_exists "spin"; then
        echo ""
        info "Remember: FTL requires gh CLI, Rust (with wasm32-wasip1 target), and Spin to be installed for full functionality"
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