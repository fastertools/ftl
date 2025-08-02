.PHONY: help dev-setup fmt lint test test-cov clean build quality

help:
	@echo "Available commands:"
	@echo "  dev-setup    Install development dependencies"
	@echo "  fmt          Format code with gofmt"
	@echo "  lint         Run linting with golangci-lint"
	@echo "  test         Run tests"
	@echo "  test-cov     Run tests with coverage"
	@echo "  clean        Clean build artifacts"
	@echo "  build        Build WebAssembly module"
	@echo "  quality      Run all quality checks"

dev-setup:
	@echo "Installing development dependencies..."
	@which golangci-lint > /dev/null || go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
	@which tinygo > /dev/null || echo "TinyGo not found. Please install from https://tinygo.org"
	go mod download
	@echo "Development environment ready!"

fmt:
	go fmt ./...

lint:
	@which golangci-lint > /dev/null || (echo "golangci-lint not found. Run 'make dev-setup' first." && exit 1)
	golangci-lint run

test:
	go test -v ./...

test-cov:
	go test -v -coverprofile=coverage.out ./...
	go tool cover -html=coverage.out -o coverage.html
	@echo "Coverage report generated: coverage.html"

clean:
	rm -f *.wasm app.wasm
	rm -f coverage.out coverage.html
	go clean

build: clean
	@echo "Building WebAssembly module..."
	@which tinygo > /dev/null || (echo "TinyGo not found. Please install from https://tinygo.org" && exit 1)
	tinygo build -target=wasip1 -gc=leaking -scheduler=none -no-debug -o app.wasm main.go
	@echo "Built: app.wasm"

# Run all quality checks
quality: fmt lint test
	@echo "All quality checks passed!"

# Verify TinyGo compatibility
verify-tinygo:
	@echo "Checking TinyGo compatibility..."
	tinygo build -target=wasip1 -gc=leaking -scheduler=none -no-debug -o /tmp/test.wasm main.go && rm /tmp/test.wasm
	@echo "TinyGo build successful!"

# Quick development cycle
dev: fmt lint test build
	@echo "Development cycle complete!"