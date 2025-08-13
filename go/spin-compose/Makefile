# Makefile for spin-compose
# Production-ready build system for a single binary distribution

# Build variables
VERSION ?= $(shell git describe --tags --always --dirty 2>/dev/null || echo "dev")
COMMIT ?= $(shell git rev-parse HEAD 2>/dev/null || echo "unknown")
BUILD_TIME ?= $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")

# Go build variables
GOOS ?= $(shell go env GOOS)
GOARCH ?= $(shell go env GOARCH)
CGO_ENABLED ?= 0

# Build flags
LDFLAGS := -s -w
LDFLAGS += -X github.com/fastertools/ftl-cli/go/spin-compose/cmd.version=$(VERSION)
LDFLAGS += -X github.com/fastertools/ftl-cli/go/spin-compose/cmd.commit=$(COMMIT)
LDFLAGS += -X github.com/fastertools/ftl-cli/go/spin-compose/cmd.buildTime=$(BUILD_TIME)

# Output
BINARY_NAME := spin-compose
OUTPUT_DIR := dist
OUTPUT_PATH := $(OUTPUT_DIR)/$(BINARY_NAME)

# Build targets
.PHONY: all build clean test lint fmt vet deps release help install

all: test build ## Run tests and build

build: deps ## Build the binary
	@echo "Building $(BINARY_NAME) $(VERSION)..."
	@mkdir -p $(OUTPUT_DIR)
	@CGO_ENABLED=$(CGO_ENABLED) GOOS=$(GOOS) GOARCH=$(GOARCH) go build \
		-ldflags "$(LDFLAGS)" \
		-o $(OUTPUT_PATH) \
		.
	@echo "Built $(OUTPUT_PATH)"

clean: ## Clean build artifacts
	@echo "Cleaning..."
	@rm -rf $(OUTPUT_DIR)
	@go clean

test: ## Run tests
	@echo "Running tests..."
	@go test -v ./...

test-coverage: ## Run tests with coverage
	@echo "Running tests with coverage..."
	@go test -v -race -coverprofile=coverage.out ./...
	@go tool cover -html=coverage.out -o coverage.html
	@echo "Coverage report generated: coverage.html"

lint: ## Run linter
	@echo "Running linter..."
	@if command -v golangci-lint >/dev/null 2>&1; then \
		golangci-lint run; \
	else \
		echo "golangci-lint not found, skipping..."; \
		echo "Install with: go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest"; \
	fi

fmt: ## Format code
	@echo "Formatting code..."
	@go fmt ./...

vet: ## Run go vet
	@echo "Running go vet..."
	@go vet ./...

deps: ## Download dependencies
	@echo "Downloading dependencies..."
	@go mod download
	@go mod tidy

# Cross-compilation targets
build-linux: ## Build for Linux
	@$(MAKE) build GOOS=linux GOARCH=amd64

build-darwin: ## Build for macOS
	@$(MAKE) build GOOS=darwin GOARCH=amd64

build-windows: ## Build for Windows
	@$(MAKE) build GOOS=windows GOARCH=amd64 BINARY_NAME=spin-compose.exe

build-all: clean ## Build for all platforms
	@echo "Building for all platforms..."
	@$(MAKE) build-linux
	@mv $(OUTPUT_DIR)/$(BINARY_NAME) $(OUTPUT_DIR)/$(BINARY_NAME)-linux-amd64
	@$(MAKE) build-darwin
	@mv $(OUTPUT_DIR)/$(BINARY_NAME) $(OUTPUT_DIR)/$(BINARY_NAME)-darwin-amd64
	@$(MAKE) build-windows
	@mv $(OUTPUT_DIR)/spin-compose.exe $(OUTPUT_DIR)/spin-compose-windows-amd64.exe
	@echo "Cross-platform binaries built in $(OUTPUT_DIR)/"

# Release targets
release: test lint build-all ## Create a release build
	@echo "Creating release $(VERSION)..."
	@cd $(OUTPUT_DIR) && \
		sha256sum * > checksums.txt
	@echo "Release artifacts:"
	@ls -la $(OUTPUT_DIR)/

# Installation targets
install: build ## Install to GOBIN
	@echo "Installing $(BINARY_NAME)..."
	@cp $(OUTPUT_PATH) $(GOBIN)/$(BINARY_NAME)
	@echo "Installed to $(GOBIN)/$(BINARY_NAME)"

install-local: build ## Install to /usr/local/bin (requires sudo)
	@echo "Installing $(BINARY_NAME) to /usr/local/bin..."
	@sudo cp $(OUTPUT_PATH) /usr/local/bin/$(BINARY_NAME)
	@echo "Installed to /usr/local/bin/$(BINARY_NAME)"

# Development targets
dev: ## Build and run in development mode
	@$(MAKE) build
	@echo "Running $(BINARY_NAME) in development mode..."
	@./$(OUTPUT_PATH) --help

# Quality assurance
qa: fmt vet lint test ## Run all quality assurance checks

# Docker targets (if needed)
docker-build: ## Build Docker image
	@echo "Building Docker image..."
	@docker build -t spin-compose:$(VERSION) .

# Git hooks
install-hooks: ## Install Git hooks
	@echo "Installing Git hooks..."
	@echo "#!/bin/sh\nmake qa" > .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "Pre-commit hook installed"

# Info targets
version: ## Show version information
	@echo "Version: $(VERSION)"
	@echo "Commit: $(COMMIT)"
	@echo "Build Time: $(BUILD_TIME)"
	@echo "GOOS: $(GOOS)"
	@echo "GOARCH: $(GOARCH)"

env: ## Show build environment
	@echo "Go version: $(shell go version)"
	@echo "GOPATH: $(GOPATH)"
	@echo "GOBIN: $(GOBIN)"
	@echo "PWD: $(PWD)"

help: ## Show this help
	@echo "Available targets:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Default target
.DEFAULT_GOAL := help