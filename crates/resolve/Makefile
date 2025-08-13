# Makefile for ftl-resolve
# GNU Make compatible build system

CARGO ?= cargo
INSTALL ?= install
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
MANDIR ?= $(PREFIX)/share/man/man1
DESTDIR ?=

# Build configuration
PROFILE ?= release
TARGET ?=
FEATURES ?=

# Derived variables
# Use workspace target directory
BUILD_DIR = ../../target/$(if $(TARGET),$(TARGET)/,)$(PROFILE)
BINARY = $(BUILD_DIR)/ftl-resolve
MAN_PAGE = man/ftl-resolve.1

# Version information
VERSION := $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

# Color output
NO_COLOR ?=
ifdef NO_COLOR
	COLOR_GREEN :=
	COLOR_RED :=
	COLOR_YELLOW :=
	COLOR_RESET :=
else
	COLOR_GREEN := \033[0;32m
	COLOR_RED := \033[0;31m
	COLOR_YELLOW := \033[0;33m
	COLOR_RESET := \033[0m
endif

.PHONY: all build test check clean install install-man uninstall dist publish publish-dry version help

# Default target
all: build test

# Help target
help:
	@echo "ftl-resolve $(VERSION) - Build System"
	@echo ""
	@echo "Usage: make [target] [VARIABLE=value ...]"
	@echo ""
	@echo "Targets:"
	@echo "  all          - Build and test (default)"
	@echo "  build        - Build the binary"
	@echo "  test         - Run all tests"
	@echo "  check        - Run linting and format checks"
	@echo "  bench        - Run benchmarks"
	@echo "  doc          - Generate documentation"
	@echo "  clean        - Remove build artifacts"
	@echo "  install      - Install to ~/.cargo/bin (user install)"
	@echo "  install-man  - Install man page to /usr/local/share/man (requires sudo)"
	@echo "  install-system - Install to /usr/local/bin (requires sudo)"
	@echo "  uninstall    - Remove installed files"
	@echo "  dist         - Create distribution tarball"
	@echo "  publish-dry  - Dry run of crate publishing (verify only)"
	@echo "  publish      - Publish crate to crates.io (requires auth)"
	@echo "  version      - Bump crate version (usage: make version BUMP=minor)"
	@echo "  coverage     - Generate test coverage report"
	@echo ""
	@echo "Variables:"
	@echo "  PROFILE      - Build profile (debug/release) [$(PROFILE)]"
	@echo "  TARGET       - Target triple for cross-compilation"
	@echo "  FEATURES     - Cargo features to enable"
	@echo "  PREFIX       - Installation prefix [$(PREFIX)]"
	@echo "  DESTDIR      - Destination directory for staged installs"
	@echo ""
	@echo "Examples:"
	@echo "  make build PROFILE=debug"
	@echo "  make install PREFIX=/opt/ftl"
	@echo "  make test FEATURES=all"

# Build the binary
build:
	@echo "$(COLOR_GREEN)Building ftl-resolve $(VERSION) ($(PROFILE))...$(COLOR_RESET)"
	$(CARGO) build --package ftl-resolve \
		$(if $(findstring release,$(PROFILE)),--release,) \
		$(if $(TARGET),--target $(TARGET),) \
		$(if $(FEATURES),--features $(FEATURES),)
	@echo "$(COLOR_GREEN)✓ Build complete: $(BINARY)$(COLOR_RESET)"

# Run tests
test: build
	@echo "$(COLOR_GREEN)Running tests...$(COLOR_RESET)"
	$(CARGO) test --package ftl-resolve \
		$(if $(findstring release,$(PROFILE)),--release,) \
		$(if $(TARGET),--target $(TARGET),) \
		$(if $(FEATURES),--features $(FEATURES),) \
		-- --nocapture
	@echo "$(COLOR_GREEN)✓ All tests passed$(COLOR_RESET)"

# Run integration tests only
test-integration: build
	@echo "$(COLOR_GREEN)Running integration tests...$(COLOR_RESET)"
	$(CARGO) test --package ftl-resolve --test '*' \
		$(if $(findstring release,$(PROFILE)),--release,) \
		$(if $(TARGET),--target $(TARGET),)

# Run unit tests only
test-unit: build
	@echo "$(COLOR_GREEN)Running unit tests...$(COLOR_RESET)"
	$(CARGO) test --package ftl-resolve --lib \
		$(if $(findstring release,$(PROFILE)),--release,) \
		$(if $(TARGET),--target $(TARGET),)

# Linting and checks
check:
	@echo "$(COLOR_GREEN)Running checks...$(COLOR_RESET)"
	$(CARGO) fmt --package ftl-resolve -- --check
	$(CARGO) clippy --package ftl-resolve --all-features -- -D warnings
	@echo "$(COLOR_GREEN)✓ All checks passed$(COLOR_RESET)"

# Format code
fmt:
	@echo "$(COLOR_GREEN)Formatting code...$(COLOR_RESET)"
	$(CARGO) fmt --package ftl-resolve

# Run benchmarks
bench:
	@echo "$(COLOR_GREEN)Running benchmarks...$(COLOR_RESET)"
	$(CARGO) bench --package ftl-resolve

# Generate documentation
doc:
	@echo "$(COLOR_GREEN)Generating documentation...$(COLOR_RESET)"
	$(CARGO) doc --package ftl-resolve --no-deps --open

# Clean build artifacts
clean:
	@echo "$(COLOR_YELLOW)Cleaning build artifacts...$(COLOR_RESET)"
	$(CARGO) clean --package ftl-resolve
	rm -rf dist/
	@echo "$(COLOR_GREEN)✓ Clean complete$(COLOR_RESET)"

# Install using cargo install (handles permissions properly)
install:
	@echo "$(COLOR_GREEN)Installing ftl-resolve using cargo install...$(COLOR_RESET)"
	$(CARGO) install --path . $(if $(FEATURES),--features $(FEATURES),)
	@echo "$(COLOR_GREEN)✓ Installation complete$(COLOR_RESET)"
	@echo "Binary installed to: $$(which ftl-resolve 2>/dev/null || echo '~/.cargo/bin/ftl-resolve')"
	@echo ""
	@echo "$(COLOR_YELLOW)If ftl-resolve is not in your PATH, add this to your shell config:$(COLOR_RESET)"
	@echo '  export PATH="$$HOME/.cargo/bin:$$PATH"'
	@echo ""
	@if [ -f "$(MAN_PAGE)" ]; then \
		echo "$(COLOR_YELLOW)To install the man page manually:$(COLOR_RESET)"; \
		echo "  sudo mkdir -p /usr/local/share/man/man1/"; \
		echo "  sudo install -m 644 $(MAN_PAGE) /usr/local/share/man/man1/"; \
		echo "  sudo gzip -f /usr/local/share/man/man1/ftl-resolve.1"; \
	fi

# Install man page only
install-man:
	@if [ -f "$(MAN_PAGE)" ]; then \
		echo "$(COLOR_GREEN)Installing man page...$(COLOR_RESET)"; \
		sudo mkdir -p /usr/local/share/man/man1/; \
		sudo install -m 644 "$(MAN_PAGE)" /usr/local/share/man/man1/; \
		if command -v gzip >/dev/null 2>&1; then \
			sudo gzip -f /usr/local/share/man/man1/ftl-resolve.1; \
		fi; \
		echo "$(COLOR_GREEN)✓ Man page installed$(COLOR_RESET)"; \
	else \
		echo "$(COLOR_RED)Error: man/ftl-resolve.1 not found$(COLOR_RESET)"; \
		exit 1; \
	fi

# Install system-wide (requires sudo)
install-system: build
	@echo "$(COLOR_GREEN)Installing ftl-resolve system-wide...$(COLOR_RESET)"
	sudo $(INSTALL) -d "$(DESTDIR)$(BINDIR)"
	sudo $(INSTALL) -m 755 "$(BINARY)" "$(DESTDIR)$(BINDIR)/ftl-resolve"
	@if [ -f "$(MAN_PAGE)" ]; then \
		echo "Installing man page..."; \
		sudo $(INSTALL) -d "$(DESTDIR)$(MANDIR)"; \
		sudo $(INSTALL) -m 644 "$(MAN_PAGE)" "$(DESTDIR)$(MANDIR)/"; \
		if command -v gzip >/dev/null 2>&1; then \
			sudo gzip -f "$(DESTDIR)$(MANDIR)/ftl-resolve.1"; \
		fi; \
	fi
	@echo "$(COLOR_GREEN)✓ System-wide installation complete$(COLOR_RESET)"
	@echo "Binary: $(DESTDIR)$(BINDIR)/ftl-resolve"
	@if [ -f "$(MAN_PAGE)" ]; then \
		echo "Man page: $(DESTDIR)$(MANDIR)/ftl-resolve.1"; \
	fi

# Uninstall
uninstall:
	@echo "$(COLOR_YELLOW)Uninstalling ftl-resolve...$(COLOR_RESET)"
	rm -f "$(DESTDIR)$(BINDIR)/ftl-resolve"
	rm -f "$(DESTDIR)$(MANDIR)/ftl-resolve.1"
	rm -f "$(DESTDIR)$(MANDIR)/ftl-resolve.1.gz"
	@echo "$(COLOR_GREEN)✓ Uninstall complete$(COLOR_RESET)"

# Create distribution tarball
dist: clean build test
	@echo "$(COLOR_GREEN)Creating distribution tarball...$(COLOR_RESET)"
	mkdir -p dist/ftl-resolve-$(VERSION)
	cp -r src/ Cargo.toml Cargo.lock README.md CHANGELOG.md LICENSE* dist/ftl-resolve-$(VERSION)/
	cp -r man/ dist/ftl-resolve-$(VERSION)/ 2>/dev/null || true
	cp $(BINARY) dist/ftl-resolve-$(VERSION)/
	cd dist && tar czf ftl-resolve-$(VERSION).tar.gz ftl-resolve-$(VERSION)
	cd dist && sha256sum ftl-resolve-$(VERSION).tar.gz > ftl-resolve-$(VERSION).tar.gz.sha256
	@echo "$(COLOR_GREEN)✓ Distribution created: dist/ftl-resolve-$(VERSION).tar.gz$(COLOR_RESET)"

# Dry run publish to crates.io (verification only)
publish-dry: check test
	@echo "$(COLOR_GREEN)Running publish dry-run...$(COLOR_RESET)"
	@echo "Checking package metadata..."
	$(CARGO) package --list --allow-dirty
	@echo ""
	@echo "$(COLOR_GREEN)Verifying package can be published...$(COLOR_RESET)"
	$(CARGO) publish --dry-run --allow-dirty
	@echo ""
	@echo "$(COLOR_GREEN)✓ Package verification successful$(COLOR_RESET)"
	@echo ""
	@echo "Current version: $(VERSION)"
	@echo "Package size: $$($(CARGO) package --list --allow-dirty 2>/dev/null | wc -l) files"
	@echo ""
	@echo "$(COLOR_YELLOW)To publish for real, run: make publish$(COLOR_RESET)"

# Publish to crates.io (requires authentication)
publish: check test
	@echo "$(COLOR_GREEN)Publishing ftl-resolve to crates.io...$(COLOR_RESET)"
	@echo ""
	@echo "Current version: $(VERSION)"
	@echo ""
	@# Check if version already exists on crates.io
	@if cargo search ftl-resolve --limit 1 | grep -q "^ftl-resolve = \"$(VERSION)\""; then \
		echo "$(COLOR_RED)Error: Version $(VERSION) is already published on crates.io$(COLOR_RESET)"; \
		echo "Please update the version in Cargo.toml before publishing."; \
		exit 1; \
	fi
	@echo "$(COLOR_YELLOW)Pre-publish checklist:$(COLOR_RESET)"
	@echo "  ✓ Tests passed"
	@echo "  ✓ Formatting checked"
	@echo "  ✓ Clippy passed"
	@echo "  ✓ Version $(VERSION) not yet published"
	@echo ""
	@echo "$(COLOR_YELLOW)This will publish ftl-resolve v$(VERSION) to crates.io$(COLOR_RESET)"
	@echo "$(COLOR_YELLOW)This action cannot be undone!$(COLOR_RESET)"
	@echo ""
	@read -p "Are you sure you want to publish? (y/N) " -n 1 -r; \
	echo ""; \
	if [[ ! $$REPLY =~ ^[Yy]$$ ]]; then \
		echo "$(COLOR_YELLOW)Publishing cancelled$(COLOR_RESET)"; \
		exit 1; \
	fi
	@echo ""
	@echo "$(COLOR_GREEN)Publishing to crates.io...$(COLOR_RESET)"
	$(CARGO) publish
	@echo ""
	@echo "$(COLOR_GREEN)✓ Successfully published ftl-resolve v$(VERSION) to crates.io!$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_YELLOW)Post-publish steps:$(COLOR_RESET)"
	@echo "  1. Create a git tag: git tag ftl-resolve-v$(VERSION)"
	@echo "  2. Push the tag: git push origin ftl-resolve-v$(VERSION)"
	@echo "  3. Create a GitHub release from the tag"
	@echo ""
	@echo "View at: https://crates.io/crates/ftl-resolve/$(VERSION)"

# Version management
version:
	@if [ -x "scripts/bump-version.sh" ]; then \
		./scripts/bump-version.sh $(BUMP); \
	else \
		echo "$(COLOR_RED)Error: scripts/bump-version.sh not found or not executable$(COLOR_RESET)"; \
		exit 1; \
	fi

# Generate test coverage
coverage:
	@echo "$(COLOR_GREEN)Generating test coverage...$(COLOR_RESET)"
	$(CARGO) llvm-cov --package ftl-resolve --html
	@echo "$(COLOR_GREEN)✓ Coverage report generated$(COLOR_RESET)"

# Development helpers
dev: build
	@echo "$(COLOR_GREEN)Running in development mode...$(COLOR_RESET)"
	$(BINARY) --help

# Run with example
example: build
	@echo "$(COLOR_GREEN)Running example transpilation...$(COLOR_RESET)"
	@echo "[project]\nname = \"example\"\n[mcp]\ngateway = \"g.wasm\"\nauthorizer = \"a.wasm\"" | $(BINARY) transpile

# Verify installation works
verify: install
	@echo "$(COLOR_GREEN)Verifying installation...$(COLOR_RESET)"
	ftl-resolve --version
	ftl-resolve --help
	@echo "$(COLOR_GREEN)✓ Verification complete$(COLOR_RESET)"

# CI targets
ci: check test
	@echo "$(COLOR_GREEN)✓ CI checks passed$(COLOR_RESET)"

.SILENT: help