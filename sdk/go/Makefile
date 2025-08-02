.PHONY: help test test-cov fmt lint clean build-examples tidy dev-deps quality verify-tinygo

help:
	@echo "Available commands:"
	@echo "  dev-deps      Install development dependencies"
	@echo "  fmt           Format code with gofmt"
	@echo "  lint          Run linting with golangci-lint"
	@echo "  test          Run tests"
	@echo "  test-cov      Run tests with coverage"
	@echo "  clean         Clean build artifacts"
	@echo "  build-examples Build example applications"
	@echo "  tidy          Run go mod tidy"
	@echo "  quality       Run all quality checks"
	@echo "  verify-tinygo Verify TinyGo compatibility"

# Run tests
test:
	go test -v ./...

# Run tests with coverage
test-cov:
	go test -v -coverprofile=coverage.out ./...
	go tool cover -html=coverage.out -o coverage.html
	@echo "Coverage report generated: coverage.html"

# Format code
fmt:
	go fmt ./...

# Run linter
lint:
	@which golangci-lint > /dev/null || (echo "golangci-lint not found. Run 'make dev-deps' first." && exit 1)
	golangci-lint run

# Clean build artifacts
clean:
	rm -rf examples/*/dist
	rm -f examples/*/*.wasm
	rm -f coverage.out coverage.html

# Build examples with TinyGo
build-examples:
	@echo "Building examples requires TinyGo to be installed"
	@which tinygo > /dev/null || (echo "TinyGo not found. Please install from https://tinygo.org" && exit 1)
	cd examples/simple && tinygo build -o simple.wasm -target=wasip1 -gc=leaking -scheduler=none -no-debug main.go

# Run go mod tidy
tidy:
	go mod tidy

# Install development dependencies
dev-deps:
	@echo "Installing development dependencies..."
	go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
	@which tinygo > /dev/null || echo "TinyGo not found. Please install from https://tinygo.org"
	@echo "Development dependencies installed!"

# Run all quality checks
quality: fmt lint test
	@echo "All quality checks passed!"

# Verify TinyGo compatibility
verify-tinygo:
	@echo "Verifying TinyGo compatibility..."
	@which tinygo > /dev/null || (echo "TinyGo not found. Please install from https://tinygo.org" && exit 1)
	@echo "Creating temporary test file..."
	@echo 'package main\nimport _ "github.com/fastertools/ftl-cli/sdk/go"\nfunc main() {}' > /tmp/tinygo_test.go
	@cd /tmp && go mod init test 2>/dev/null || true
	@cd /tmp && go get github.com/fastertools/ftl-cli/sdk/go@latest 2>/dev/null || true
	@tinygo build -target=wasip1 -gc=leaking -scheduler=none -no-debug -o /tmp/test.wasm /tmp/tinygo_test.go && rm /tmp/test.wasm /tmp/tinygo_test.go
	@echo "TinyGo compatibility verified!"