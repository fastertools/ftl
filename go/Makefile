# FTL CLI Makefile

# Variables
BINARY_NAME=ftl
VERSION?=dev
COMMIT=$(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_DATE=$(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
LDFLAGS=-ldflags "-X github.com/fastertools/ftl-cli/go/ftl/cmd.version=$(VERSION) -X github.com/fastertools/ftl-cli/go/ftl/cmd.commit=$(COMMIT) -X github.com/fastertools/ftl-cli/go/ftl/cmd.buildDate=$(BUILD_DATE)"

# Go parameters
GOCMD=go
GOBUILD=$(GOCMD) build
GOTEST=$(GOCMD) test
GOGET=$(GOCMD) get
GOMOD=$(GOCMD) mod
GOINSTALL=$(GOCMD) install

# Directories
FTL_DIR=./ftl
SHARED_DIR=./shared

.PHONY: all build clean test coverage install uninstall help

## help: Show this help message
help:
	@echo 'Usage:'
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':' | sed -e 's/^/ /'

## all: Build FTL
all: build

## build: Build the FTL CLI binary
build:
	@echo "Building FTL CLI..."
	cd $(FTL_DIR) && $(GOBUILD) $(LDFLAGS) -o ../bin/$(BINARY_NAME) -v

## install: Install FTL CLI to system
install: build
	@echo "Installing FTL CLI to $$(go env GOPATH)/bin..."
	@mkdir -p $$(go env GOPATH)/bin
	@cp bin/$(BINARY_NAME) $$(go env GOPATH)/bin/
	@echo "✅ FTL CLI installed to $$(go env GOPATH)/bin/$(BINARY_NAME)"
	@echo ""
	@echo "Make sure $$(go env GOPATH)/bin is in your PATH:"
	@echo '  export PATH=$$PATH:$$(go env GOPATH)/bin'

## uninstall: Remove FTL CLI from system
uninstall:
	@echo "Removing FTL CLI..."
	@rm -f $$(go env GOPATH)/bin/$(BINARY_NAME)
	@echo "✅ FTL CLI removed"

## test: Run all tests
test:
	@echo "Testing shared libraries..."
	cd $(SHARED_DIR) && $(GOTEST) -v \
		./types \
		./spin \
		./ftl \
		./auth \
		-cover
	@echo ""
	@echo "Testing FTL CLI..."
	cd $(FTL_DIR) && $(GOTEST) -v ./cmd/... ./pkg/... -cover

## test-short: Run tests without verbose output
test-short:
	@cd $(SHARED_DIR) && $(GOTEST) ./types ./spin ./ftl ./auth -cover
	@cd $(FTL_DIR) && $(GOTEST) ./cmd/... ./pkg/... -cover

## coverage: Generate test coverage report
coverage:
	@echo "Generating combined coverage report..."
	@cd $(FTL_DIR) && $(GOTEST) -coverprofile=coverage.out \
		./cmd/... \
		./pkg/... \
		../shared/types \
		../shared/spin \
		../shared/ftl \
		../shared/auth 2>/dev/null || true
	@cd $(FTL_DIR) && $(GOCMD) tool cover -html=coverage.out -o coverage.html
	@cd $(FTL_DIR) && $(GOCMD) tool cover -func=coverage.out | tail -1
	@echo "Coverage report generated: ftl/coverage.html"

## coverage-detailed: Generate detailed coverage with percentages per package
coverage-detailed:
	@echo "Generating detailed coverage report..."
	@cd $(FTL_DIR) && $(GOTEST) -coverprofile=coverage.out \
		./cmd/... \
		./pkg/... \
		../shared/types \
		../shared/spin \
		../shared/ftl \
		../shared/auth 2>/dev/null || true
	@echo ""
	@echo "Coverage by package:"
	@cd $(FTL_DIR) && $(GOCMD) tool cover -func=coverage.out | grep -E "github.com/fastertools" | grep -v ".go:" || true
	@echo ""
	@echo "Overall coverage:"
	@cd $(FTL_DIR) && $(GOCMD) tool cover -func=coverage.out | tail -1

## clean: Remove build artifacts
clean:
	@echo "Cleaning up..."
	@rm -rf bin/
	@rm -f $(SHARED_DIR)/coverage.out $(SHARED_DIR)/coverage.html
	@rm -f $(FTL_DIR)/coverage.out $(FTL_DIR)/coverage.html
	@echo "✅ Cleaned"

## deps: Download dependencies
deps:
	@echo "Downloading dependencies..."
	cd $(SHARED_DIR) && $(GOMOD) download
	cd $(FTL_DIR) && $(GOMOD) download

## tidy: Tidy go modules
tidy:
	@echo "Tidying modules..."
	cd $(SHARED_DIR) && $(GOMOD) tidy
	cd $(FTL_DIR) && $(GOMOD) tidy

## fmt: Format code
fmt:
	@echo "Formatting code..."
	$(GOCMD) fmt ./...

## vet: Run go vet
vet:
	@echo "Running go vet..."
	cd $(SHARED_DIR) && $(GOCMD) vet ./...
	cd $(FTL_DIR) && $(GOCMD) vet ./...

## generate: Regenerate API client from OpenAPI spec
generate:
	@echo "Regenerating API client from OpenAPI spec..."
	@rm -f $(SHARED_DIR)/api/client.gen.go
	cd $(SHARED_DIR)/api && $(GOCMD) generate ./...
	@echo "✅ API client regenerated"

## run: Run FTL CLI directly
run:
	cd $(FTL_DIR) && $(GOCMD) run . $(ARGS)

# Default target
.DEFAULT_GOAL := help