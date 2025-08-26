# FTL - Polyglot WebAssembly MCP Platform
# Main orchestration Makefile

.PHONY: all help build test clean install setup-browser-tests setup-all

# Default to showing help
help:
	@echo "FTL - Fast, polyglot toolkit for building MCP tools on WebAssembly"
	@echo ""
	@echo "Available targets:"
	@echo ""
	@echo "  Core Commands:"
	@echo "    build         - Build FTL CLI (Go)"
	@echo "    test          - Run all tests (Go CLI + Rust SDKs)"
	@echo "    install       - Install FTL CLI to system"
	@echo "    clean         - Clean all build artifacts"
	@echo ""
	@echo "  Component Commands:"
	@echo "    build-components  - Build WebAssembly components"
	@echo "    test-components   - Test WebAssembly components"
	@echo ""
	@echo "  Development Commands:"
	@echo "    generate-api  - Generate API client from OpenAPI spec"
	@echo "    generate-templ- Generate templ templates"
	@echo "    generate-ui   - Generate UI constants from CUE modules"
	@echo "    fmt           - Format all code (Go + Rust)"
	@echo "    lint          - Lint all code (Go + Rust)"
	@echo "    coverage      - Generate test coverage reports"
	@echo ""
	@echo "  Setup Commands:"
	@echo "    setup-browser-tests - Install browser testing dependencies"
	@echo "    setup-all     - Setup all project dependencies"
	@echo ""
	@echo "  Testing Commands:"
	@echo "    test          - Run all tests (Go + MCP + Console + Browser)"
	@echo "    test-go       - Run Go unit tests only"
	@echo "    test-mcp      - Test MCP server functionality"
	@echo "    test-console  - Test console server functionality"
	@echo "    test-browser  - Run browser/playwright tests"
	@echo "    test-browser-headed - Run browser tests with visible browser"
	@echo "    test-browser-debug  - Run browser tests in debug mode"
	@echo "    test-new-user - Test new user centralized experience"
	@echo ""
	@echo "  Quick Commands:"
	@echo "    dev           - Quick development build and test"
	@echo "    all           - Build everything (CLI + components)"

# Generate API client from OpenAPI spec
generate-api:
	@echo "🔄 Generating API client from OpenAPI spec..."
	@oapi-codegen -package api -generate types,client -o internal/api/client.gen.go internal/api/openapi.json
	@echo "✅ API client generated: internal/api/client.gen.go"

# Generate templ templates
generate-templ:
	@echo "🔄 Generating templ templates..."
	@if command -v templ >/dev/null 2>&1; then \
		templ generate; \
		echo "✅ Templ templates generated"; \
	else \
		echo "⚠️  templ not installed"; \
		echo "   Install with: go install github.com/a-h/templ/cmd/templ@latest"; \
	fi

# Generate UI constants from CUE
generate-ui:
	@echo "🔄 Generating UI constants from CUE modules..."
	@cd ui && go run generate_v2.go
	@echo "✅ UI constants generated: ui/gen/"

# Build FTL CLI (Go)
build:
	@echo "🔨 Building FTL CLI..."
	@go build -ldflags "-X github.com/fastertools/ftl/internal/cli.version=$$(git describe --tags --always) \
		-X github.com/fastertools/ftl/internal/cli.commit=$$(git rev-parse --short HEAD) \
		-X github.com/fastertools/ftl/internal/cli.buildDate=$$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
		-o bin/ftl ./cmd/ftl
	@echo "✅ FTL CLI built: bin/ftl"

# Test Go CLI
test-cli:
	@echo "🧪 Testing FTL CLI..."
	@go test -v ./...

# Test Rust SDKs
test-sdk:
	@echo "🧪 Testing Rust SDKs..."
	@cd sdk/rust && cargo test --target $(shell rustc -vV | sed -n 's/host: //p')
	@cd sdk/rust-macros && cargo test --target $(shell rustc -vV | sed -n 's/host: //p')

# Build WebAssembly components
build-components:
	@echo "🔨 Building WebAssembly components..."
	@# With .cargo/config.toml, wasm32-wasip1 is the default target
	@cargo build --workspace --release
	@echo "✅ Components built in target/wasm32-wasip1/release/"

# Test WebAssembly components
test-components:
	@echo "🧪 Testing WebAssembly components..."
	@if command -v spin >/dev/null 2>&1; then \
		cd components/mcp-authorizer && spin test; \
		cd ../mcp-gateway && spin test; \
	else \
		echo "⚠️  spin not installed"; \
		echo "   Install from: https://developer.fermyon.com/spin/install"; \
	fi

# Run all tests
test: test-cli test-sdk test-components

# Format all code
fmt:
	@echo "🎨 Formatting code..."
	@echo "  Formatting Go code..."
	@go fmt ./...
	@echo "  Formatting Rust code..."
	@cargo fmt --all

# Lint all code
lint:
	@echo "🔍 Linting code..."
	@echo "  Linting Go code..."
	@go vet ./...
	@if command -v golangci-lint >/dev/null 2>&1; then \
		golangci-lint run ./...; \
	fi
	@echo "  Linting Rust code..."
	@cargo clippy --all-targets --all-features -- -D warnings

# Generate coverage reports
coverage:
	@echo "📊 Generating coverage reports..."
	@echo "  Go coverage..."
	@go test -coverprofile=coverage.out ./...
	@go tool cover -html=coverage.out -o coverage-go.html
	@go tool cover -func=coverage.out | tail -1
	@echo "  Coverage report: coverage-go.html"

# Install FTL CLI
install: build
	@echo "📦 Installing FTL CLI..."
	@mkdir -p ~/.local/bin
	@cp bin/ftl ~/.local/bin/
	@echo "✅ Installed to ~/.local/bin/ftl"
	@echo ""
	@echo "Make sure ~/.local/bin is in your PATH:"
	@echo '  export PATH=$$HOME/.local/bin:$$PATH'

# Install to system location (requires sudo)
install-system: build
	@echo "📦 Installing FTL CLI to system..."
	@sudo cp bin/ftl /usr/local/bin/
	@echo "✅ Installed to /usr/local/bin/ftl"

# Clean all build artifacts
clean: clean-test-data
	@echo "🧹 Cleaning build artifacts..."
	@rm -rf bin/ target/ coverage*.out coverage*.html
	@rm -rf node_modules/ playwright-report/ test-results/
	@go clean -cache -testcache
	@cargo clean
	@echo "✅ Clean complete"

# Quick development cycle
dev: fmt build test-cli
	@echo "✅ Development build complete"

# Build everything
all: build build-components
	@echo "✅ All components built successfully"

# Test targets
test-go:
	@echo "🧪 Running Go unit tests..."
	@go test -timeout=2m -short ./internal/... -v
	@echo "✅ Go tests completed"

test-mcp: build
	@echo "🧪 Testing MCP server mode..."
	@echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' | ./bin/ftl dev mcp | grep -q "mcp-server" && echo "✅ MCP mode works" || echo "❌ MCP mode failed"

test-console: build kill
	@echo "🧪 Testing console server functionality..."
	@./bin/ftl dev console --port 8080 2>&1 | head -20 | grep -q "Starting server on port" && echo "✅ Console mode starts" || echo "❌ Console mode failed"

test-browser: build kill setup-browser-tests clean-test-data
	@echo "🧪 Running browser tests with standardized test projects file..."
	@rm -f .e2e-projects.json
	@PROJECTS_FILE=.e2e-projects.json ./bin/ftl dev console --port 8080 > test_server.log 2>&1 &
	@sleep 3
	@npx playwright test --config=playwright.config.js || true
	@pkill -f "ftl dev console" || true
	@$(MAKE) clean-test-data
	@echo "Tests completed"

test-browser-headed: build kill clean-test-data
	@echo "🧪 Starting FTL dev console for headed browser tests..."
	@PROJECTS_FILE=test_projects.json ./bin/ftl dev console --port 8080 > test_server.log 2>&1 &
	@sleep 3
	@echo "Running tests with visible browser..."
	@npx playwright test --headed
	@pkill -f "ftl dev console" || true
	@$(MAKE) clean-test-data

test-browser-debug: build kill clean-test-data
	@echo "🧪 Starting FTL dev console for debug tests..."
	@PROJECTS_FILE=test_projects.json ./bin/ftl dev console --port 8080 > test_server.log 2>&1 &
	@sleep 3
	@echo "Running tests in debug mode..."
	@npx playwright test --debug
	@pkill -f "ftl dev console" || true
	@$(MAKE) clean-test-data

# Run FTL dev console with test data
run-test-console: build clean-test-data
	@echo "🚀 Starting FTL dev console with test data..."
	@PROJECTS_FILE=test_projects.json ./bin/ftl dev console

# Run FTL dev console with centralized projects file (no PROJECTS_FILE env var)
run-console-centralized: build kill
	@echo "🚀 Starting FTL dev console with centralized projects file..."
	@./bin/ftl dev console

# Kill all FTL processes
kill:
	@echo "🔪 Killing all FTL processes..."
	@pkill -f "ftl dev" || true
	@lsof -ti:8080,8081,8082,8083,8084,8085,8086,8087,8088,8089 | xargs kill -9 2>/dev/null || true
	@echo "✅ All FTL processes killed"

# Setup browser testing dependencies
setup-browser-tests:
	@echo "📦 Setting up browser test dependencies..."
	@if command -v npm >/dev/null 2>&1; then \
		npm run setup; \
		echo "✅ Browser test dependencies ready"; \
	else \
		echo "⚠️  npm not installed"; \
		echo "   Install Node.js from: https://nodejs.org/"; \
	fi

# Setup all project dependencies
setup-all: setup-browser-tests
	@echo "🔧 All dependencies configured"

# Clean test data
clean-test-data:
	@echo "🧹 Cleaning E2E test data..."
	@rm -f test_projects.json
	@rm -f .e2e-projects.json
	@rm -rf .e2e-projects/
	@echo "✅ E2E test data cleaned"

# Clean centralized projects data for new user testing
clean-centralized-data:
	@echo "🧹 Cleaning centralized projects data..."
	@rm -rf "$(HOME)/Library/Application Support/ftl/"
	@echo "✅ Centralized projects data cleaned"

# Test new user centralized experience (no PROJECTS_FILE env var)
test-new-user: build kill clean-centralized-data setup-browser-tests
	@echo "🧪 Testing new user centralized experience..."
	@./bin/ftl dev console --port 8082 > test_centralized_server.log 2>&1 &
	@echo "⏳ Waiting for server to start..."
	@sleep 3
	@cd e2e-tests && npx playwright test e2e/specs/new-user-centralized.spec.js
	@echo "🛑 Stopping centralized test server..."
	@pkill -f "ftl dev console --port 8082" || true
	@echo "🧹 Cleaning up centralized test..."
	@rm -f test_centralized_server.log
	@$(MAKE) clean-centralized-data

# Debug centralized file creation (temporary)
debug-centralized-file: build kill clean-centralized-data
	@echo "🧪 Debug: Testing centralized file creation..."
	@./bin/ftl dev console --port 8082 > test_centralized_server.log 2>&1 &
	@echo "⏳ Waiting for server to start..."
	@sleep 5
	@echo "📋 Checking if file was created..."
	@ls -la "$(HOME)/Library/Application Support/ftl/" || echo "❌ Directory does not exist"
	@cat "$(HOME)/Library/Application Support/ftl/projects.json" || echo "❌ File does not exist"
	@echo "📄 Server logs:"
	@cat test_centralized_server.log
	@echo "🛑 Stopping test server..."
	@pkill -f "ftl dev console --port 8082" || true
	@rm -f test_centralized_server.log

# Test all functionality
test: test-go test-mcp test-console test-browser

.DEFAULT_GOAL := help