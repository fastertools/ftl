.PHONY: test fmt lint clean build-examples

# Run tests
test:
	go test -v ./...

# Format code
fmt:
	go fmt ./...

# Run linter
lint:
	golangci-lint run

# Clean build artifacts
clean:
	rm -rf examples/*/dist
	rm -f examples/*/*.wasm

# Build examples with TinyGo
build-examples:
	@echo "Building examples requires TinyGo to be installed"
	@which tinygo > /dev/null || (echo "TinyGo not found. Please install from https://tinygo.org" && exit 1)
	cd examples/simple && tinygo build -o simple.wasm -target=wasi main.go

# Run go mod tidy
tidy:
	go mod tidy

# Install development dependencies
dev-deps:
	go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest