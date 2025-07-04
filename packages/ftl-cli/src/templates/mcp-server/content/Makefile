.PHONY: build
build:
	cd handler && cargo component build --release

.PHONY: run
run: build
	spin up

.PHONY: test
test:
	@echo "Testing MCP server..."
	@echo "1. List tools:"
	@curl -s -X POST http://localhost:3000/mcp \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}' | jq .
	@echo ""
	@echo "2. Call example tool:"
	@curl -s -X POST http://localhost:3000/mcp \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "example_tool", "arguments": "{\"message\": \"Hello from test!\"}"}, "id": 2}' | jq .

.PHONY: deploy
deploy: build
	spin cloud deploy

.PHONY: clean
clean:
	cd handler && cargo clean

.PHONY: help
help:
	@echo "Available commands:"
	@echo "  make build  - Build the handler component"
	@echo "  make run    - Build and run the MCP server"
	@echo "  make test   - Test the MCP server (requires server running)"
	@echo "  make deploy - Deploy to Spin Cloud"
	@echo "  make clean  - Clean build artifacts"