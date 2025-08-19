package scaffold

// Language type constraint
#Language: "rust" | "typescript" | "python" | "go"

// Component name validation
#ComponentName: string & =~"^[a-z][a-z0-9-]*$"

// Base component structure
#Component: {
	name:     #ComponentName
	language: #Language
	
	// Build configuration (used in ftl.yaml)
	build: #BuildConfig
	
	// Files to generate
	files: [string]: string
}

// Build configuration structure
#BuildConfig: {
	command: string
	watch: [...string]
}

// Rust component template
#RustComponent: #Component & {
	language: "rust"
	name: string
	
	build: {
		command: "make build"
		watch: ["src/**/*.rs", "Cargo.toml"]
	}
	
	files: {
		"README.md": """
			# \(name)
			
			MCP component authored in Rust.
			
			## Development
			
			### Build the component
			```bash
			make build
			```
			
			### Run tests
			```bash
			make test
			```
			
			### Clean build artifacts
			```bash
			make clean
			```
			
			## Integration with FTL
			
			This component has been automatically added to your `ftl.yaml` configuration.
			
			To synthesize and run:
			```bash
			# From the project root
			ftl synth    # Generates spin.toml
			ftl up       # Runs the application
			```
			
			## Configuration
			
			The component configuration in `ftl.yaml`:
			- `source`: Path to the compiled WASM file
			- `build.command`: Build command (make build)
			- `build.watch`: Files to watch for auto-rebuild
			
			## Adding Tools
			
			Edit `src/lib.rs` and add new tools in the `tools!` macro:
			```rust
			tools! {
			    fn your_tool(input: YourInput) -> ToolResponse {
			        // Tool implementation
			    }
			}
			```
			"""
		
		"Cargo.toml": """
			[package]
			name = "\(name)"
			version = "0.1.0"
			edition = "2024"
			rust-version = "1.89"
			license = "Apache-2.0"
			description = "MCP component authored in Rust"

			[dependencies]
			ftl-sdk = { version = "\(_versions.rust)", features = ["macros"] }
			serde = { version = "1.0", features = ["derive"] }
			serde_json = "1.0"
			schemars = "1.0.4"
			spin-sdk = "4.0.0"

			[lib]
			crate-type = ["cdylib"]

			[lints.rust]
			unsafe_code = "forbid"

			[lints.clippy]
			# Deny categories
			correctness = { level = "deny", priority = -1 }
			suspicious = { level = "deny", priority = -1 }
			perf = { level = "deny", priority = -1 }

			# Warn categories  
			complexity = { level = "warn", priority = -1 }
			style = { level = "warn", priority = -1 }
			pedantic = { level = "warn", priority = -1 }
			"""
		
		"Makefile": """
			.PHONY: build clean test

			build:
			\tcargo build --release --target wasm32-wasip1
			\tcp target/wasm32-wasip1/release/\(name).wasm \(name).wasm

			clean:
			\tcargo clean
			\trm -f \(name).wasm

			test:
			\tcargo test
			"""
		
		"src/lib.rs": """
			use ftl_sdk::{tools, text, ToolResponse};
			use serde::Deserialize;
			use schemars::JsonSchema;

			#[derive(Deserialize, JsonSchema)]
			struct ExampleToolInput {
			    /// The input message to process
			    message: String,
			}

			tools! {
			    /// An example tool that processes messages
			    fn example_tool(input: ExampleToolInput) -> ToolResponse {
			        // TODO: Implement your tool logic here
			        text!("Processed: {}", input.message)
			    }
			    
			    // Add more tools here as needed:
			    // fn another_tool(input: AnotherInput) -> ToolResponse {
			    //     text!("Another tool response")
			    // }
			}
			"""
		
		".gitignore": """
			/target
			Cargo.lock
			*.wasm
			"""
	}
}

// TypeScript component template
#TypeScriptComponent: #Component & {
	language: "typescript"
	name: string
	
	build: {
		command: "make build"
		watch: ["src/**/*.ts", "src/**/*.js", "package.json", "tsconfig.json"]
	}
	
	files: {
		"README.md": """
			# \(name)
			
			MCP component authored in TypeScript.
			
			## Development
			
			### Install dependencies
			```bash
			npm install
			```
			
			### Build the component
			```bash
			make build
			# or
			npm run build
			```
			
			### Type checking
			```bash
			npm run typecheck
			```
			
			## Integration with FTL
			
			This component has been automatically added to your `ftl.yaml` configuration.
			
			To synthesize and run:
			```bash
			# From the project root
			ftl synth    # Generates spin.toml
			ftl up       # Runs the application
			```
			
			## Adding Tools
			
			Edit `src/index.ts` and add new tools in the `createTools` call:
			```typescript
			const handle = createTools({
			  yourTool: {
			    description: 'Tool description',
			    inputSchema: z.toJSONSchema(YourSchema),
			    handler: async (input) => {
			      // Tool implementation
			      return ToolResponse.text('Response')
			    }
			  }
			})
			```
			"""
		
		"package.json": """
			{
			  "name": "\(name)",
			  "version": "0.1.0",
			  "description": "MCP component authored in TypeScript",
			  "main": "index.js",
			  "scripts": {
			    "build": "npm run typecheck && esbuild src/index.ts --bundle --outfile=build/bundle.js --format=esm --platform=browser --external:node:* && mkdir -p dist && j2w -i build/bundle.js -o dist/\(name).wasm",
			    "typecheck": "tsc --noEmit"
			  },
			  "keywords": ["mcp", "ftl", "tool"],
			  "license": "Apache-2.0",
			  "devDependencies": {
			    "esbuild": "^0.19.0",
			    "typescript": "^5.8.3"
			  },
			  "dependencies": {
			    "@spinframework/build-tools": "^1.0.1",
			    "@spinframework/wasi-http-proxy": "^1.0.0",
			    "ftl-sdk": "^\(_versions.typescript)",
			    "zod": "^4.0.3"
			  }
			}
			"""
		
		"tsconfig.json": """
			{
			  "compilerOptions": {
			    "target": "ES2020",
			    "module": "ESNext",
			    "lib": ["ES2020"],
			    "moduleResolution": "node",
			    "strict": true,
			    "esModuleInterop": true,
			    "skipLibCheck": true,
			    "forceConsistentCasingInFileNames": true,
			    "resolveJsonModule": true,
			    "noEmit": true
			  },
			  "include": ["src/**/*"],
			  "exclude": ["node_modules", "dist", "build"]
			}
			"""
		
		
		"Makefile": """
			.PHONY: build clean test install format lint

			install:
			\tnpm install

			build: install
			\tnpm run build

			clean:
			\trm -rf dist node_modules build

			test: install
			\tnpm test

			format: install
			\tnpm run format || echo "No format script defined"

			lint: install
			\tnpm run lint || echo "No lint script defined"

			dev: install format lint test
			\t@echo "Development checks passed!"
			"""
		
		"src/index.ts": """
			import { createTools, ToolResponse } from 'ftl-sdk'
			import * as z from 'zod'

			// Define the schema using Zod
			const ExampleToolSchema = z.object({
			  message: z.string().describe('The input message to process')
			})

			const handle = createTools({
			  // Replace 'exampleTool' with your actual tool name
			  exampleTool: {
			    description: 'An example tool that processes messages',
			    inputSchema: z.toJSONSchema(ExampleToolSchema),
			    handler: async (input: z.infer<typeof ExampleToolSchema>) => {
			      // TODO: Implement your tool logic here
			      return ToolResponse.text(`Processed: ${input.message}`)
			    }
			  }
			  
			  // Add more tools here as needed:
			  // anotherTool: {
			  //   description: 'Another tool description',
			  //   inputSchema: z.toJSONSchema(AnotherSchema),
			  //   handler: async (input: z.infer<typeof AnotherSchema>) => {
			  //     return ToolResponse.text('Another response')
			  //   }
			  // }
			})

			//@ts-ignore
			addEventListener('fetch', (event: FetchEvent) => {
			  event.respondWith(handle(event.request))
			})
			"""
		
		".gitignore": """
			node_modules/
			dist/
			build/
			*.wasm
			.env
			"""
	}
}

// Python component template
#PythonComponent: #Component & {
	language: "python"
	name: string
	
	build: {
		command: "make build"
		watch: ["src/**/*.py", "pyproject.toml"]
	}
	
	files: {
		"README.md": """
			# \(name)
			
			MCP component authored in Python.
			
			## Development
			
			### Setup development environment
			```bash
			make install-dev
			# or
			pip install -e ".[dev]"
			```
			
			### Build the component
			```bash
			make build
			```
			
			### Run tests
			```bash
			make test
			```
			
			### Code quality
			```bash
			make format    # Format with black
			make lint      # Lint with ruff
			make type-check # Type check with mypy
			```
			
			## Integration with FTL
			
			This component has been automatically added to your `ftl.yaml` configuration.
			
			To synthesize and run:
			```bash
			# From the project root
			ftl synth    # Generates spin.toml
			ftl up       # Runs the application
			```
			
			## Adding Tools
			
			Edit `src/main.py` and add new tools in the `create_tools` call:
			```python
			handle = create_tools({
			    "yourTool": {
			        "description": "Tool description",
			        "input_schema": YourInput.model_json_schema(),
			        "handler": your_tool_function
			    }
			})
			```
			"""
		
		"pyproject.toml": """
			[project]
			name = "\(name)"
			version = "0.1.0"
			description = "MCP component authored in Python"
			readme = "README.md"
			requires-python = ">=3.10"
			dependencies = [
			    "ftl-sdk==\(_versions.python)",
			    # Add your project dependencies here
			]

			[project.optional-dependencies]
			dev = [
			    "pytest>=7.0",
			    "pytest-cov>=4.0",
			    "pytest-asyncio>=0.21",
			    "black>=23.0",
			    "ruff>=0.1.0",
			    "mypy>=1.0",
			    "pre-commit>=3.0",
			]

			[build-system]
			requires = ["hatchling"]
			build-backend = "hatchling.build"

			[tool.hatch.build]
			exclude = [
			    "*.wasm",
			    "__pycache__",
			    "tests/",
			    ".ruff_cache/",
			    ".mypy_cache/",
			    ".pytest_cache/",
			    ".coverage",
			]

			[tool.hatch.build.targets.wheel]
			packages = ["src"]

			# Black configuration
			[tool.black]
			line-length = 88
			target-version = ['py310']

			# Ruff configuration (fast Python linter)
			[tool.ruff]
			line-length = 88
			target-version = "py310"
			select = [
			    "E",   # pycodestyle errors
			    "W",   # pycodestyle warnings
			    "F",   # pyflakes
			    "I",   # isort
			    "B",   # flake8-bugbear
			    "C4",  # flake8-comprehensions
			    "UP",  # pyupgrade
			]
			ignore = []
			fix = true

			# MyPy configuration
			[tool.mypy]
			python_version = "3.10"
			warn_return_any = true
			warn_unused_configs = true
			disallow_untyped_defs = true
			check_untyped_defs = true
			no_implicit_optional = true
			warn_redundant_casts = true
			warn_unused_ignores = true

			# Pytest configuration
			[tool.pytest.ini_options]
			testpaths = ["tests"]
			python_files = ["test_*.py", "*_test.py"]
			addopts = "--cov=src --cov-report=term-missing"

			# Coverage configuration
			[tool.coverage.run]
			source = ["src"]
			omit = ["*/tests/*", "*/__pycache__/*"]

			[tool.coverage.report]
			exclude_lines = [
			    "pragma: no cover",
			    "def __repr__",
			    "raise AssertionError",
			    "raise NotImplementedError",
			    "if __name__ == .__main__.:",
			]
			"""
		
		"Makefile": """
			.PHONY: help install install-dev format lint type-check test test-cov clean build

			help:
			\t@echo "Available commands:"
			\t@echo "  install      Install project dependencies"
			\t@echo "  install-dev  Install project with development dependencies"
			\t@echo "  format       Format code with black"
			\t@echo "  lint         Run linting with ruff"
			\t@echo "  type-check   Run type checking with mypy"
			\t@echo "  test         Run tests"
			\t@echo "  test-cov     Run tests with coverage"
			\t@echo "  clean        Clean build artifacts"
			\t@echo "  build        Build WebAssembly module"

			install:
			\tpip install -e .

			install-dev:
			\tpip install -e ".[dev]"
			\t@echo "Installing componentize-py for WebAssembly builds..."
			\tpip install componentize-py

			format:
			\tblack src tests

			lint:
			\truff check src tests --fix

			type-check:
			\tmypy src

			test:
			\tpytest

			test-cov:
			\tpytest --cov=src --cov-report=term-missing --cov-report=html

			clean:
			\trm -rf build dist *.egg-info
			\trm -rf .coverage htmlcov .pytest_cache .mypy_cache .ruff_cache
			\trm -f app.wasm
			\tfind . -type d -name __pycache__ -exec rm -rf {} +
			\tfind . -type f -name "*.pyc" -delete

			build: clean
			\t@echo "Building WebAssembly module..."
			\t@if [ ! -d "venv" ]; then \\
			\t\techo "Creating Python virtual environment..."; \\
			\t\tpython3 -m venv venv; \\
			\tfi
			\t@echo "Installing dependencies..."
			\t@. venv/bin/activate && pip install --upgrade pip --quiet
			\t@. venv/bin/activate && pip install componentize-py --quiet
			\t@. venv/bin/activate && pip install -e . --quiet
			\t@. venv/bin/activate && pip install ftl-sdk --quiet
			\t@echo "Building WASM component..."
			\t@. venv/bin/activate && componentize-py -w spin-http componentize src.main -p . -o app.wasm
			\t@echo "✓ Build successful!"

			# Development workflow
			dev: install-dev
			\t@echo "Development environment ready!"
			\t@echo "Run 'make test' to run tests"
			\t@echo "Run 'make format' to format code"
			\t@echo "Run 'make lint' to run linting"
			\t@echo "Run 'make build' to build WebAssembly module"
			"""
		
		"src/__init__.py": ""
		
		"src/main.py": """
			from ftl_sdk import create_tools, ToolResponse
			from pydantic import BaseModel, Field
			from typing import Dict, Any

			class ExampleToolInput(BaseModel):
			    \"\"\"Input for the example tool\"\"\"
			    message: str = Field(description="The input message to process")

			async def example_tool(input: ExampleToolInput) -> ToolResponse:
			    \"\"\"An example tool that processes messages\"\"\"
			    # TODO: Implement your tool logic here
			    return ToolResponse.text(f"Processed: {input.message}")

			# Register all tools
			handle = create_tools({
			    "exampleTool": {
			        "description": "An example tool that processes messages",
			        "input_schema": ExampleToolInput.model_json_schema(),
			        "handler": example_tool
			    }
			    
			    # Add more tools here as needed:
			    # "anotherTool": {
			    #     "description": "Another tool description",
			    #     "input_schema": AnotherInput.model_json_schema(),
			    #     "handler": another_tool
			    # }
			})

			# Export the handler for the WASM runtime
			__all__ = ["handle"]
			"""
		
		"tests/__init__.py": ""
		
		"tests/test_main.py": """
			import pytest
			from src.main import example_tool, ExampleToolInput

			@pytest.mark.asyncio
			async def test_example_tool():
			    input_data = ExampleToolInput(message="Hello, World!")
			    response = await example_tool(input_data)
			    assert response.text == "Processed: Hello, World!"
			"""
		
		".gitignore": """
			__pycache__/
			*.py[cod]
			*$py.class
			*.wasm
			.pytest_cache/
			.mypy_cache/
			.ruff_cache/
			*.egg-info/
			dist/
			build/
			.env
			venv/
			.coverage
			htmlcov/
			"""
	}
}

// Go component template
#GoComponent: #Component & {
	language: "go"
	name: string
	
	build: {
		command: "make build"
		watch: ["*.go", "go.mod"]
	}
	
	files: {
		"README.md": """
			# \(name)
			
			MCP component authored in Go.
			
			## Development
			
			### Setup development environment
			```bash
			make dev-setup
			```
			
			### Build the component
			```bash
			make build
			```
			
			### Run tests
			```bash
			make test
			```
			
			### Code quality
			```bash
			make fmt     # Format code
			make lint    # Run linter
			make quality # Run all quality checks
			```
			
			## Integration with FTL
			
			This component has been automatically added to your `ftl.yaml` configuration.
			
			To synthesize and run:
			```bash
			# From the project root
			ftl synth    # Generates spin.toml
			ftl up       # Runs the application
			```
			
			## TinyGo Requirements
			
			This component requires TinyGo for WebAssembly compilation.
			Install from: https://tinygo.org
			
			## Adding Tools
			
			Edit `main.go` and add new tools in the `ftl.Handle` call:
			```go
			ftl.Handle(ftl.Tools{
			    "yourTool": {
			        Description: "Tool description",
			        InputSchema: YourInput{},
			        Handler:     YourToolFunction,
			    },
			})
			```
			"""
		
		"go.mod": """
			module github.com/example/\(name)

			go 1.23

			require (
			\tgithub.com/fastertools/ftl-cli/sdk/go v\(_versions.go)
			)
			"""
		
		"Makefile": """
			.PHONY: help dev-setup fmt lint test test-cov clean build quality

			help:
			\t@echo "Available commands:"
			\t@echo "  dev-setup    Install development dependencies"
			\t@echo "  fmt          Format code with gofmt"
			\t@echo "  lint         Run linting with golangci-lint"
			\t@echo "  test         Run tests"
			\t@echo "  test-cov     Run tests with coverage"
			\t@echo "  clean        Clean build artifacts"
			\t@echo "  build        Build WebAssembly module"
			\t@echo "  quality      Run all quality checks"

			dev-setup:
			\t@echo "Installing development dependencies..."
			\t@which golangci-lint > /dev/null || go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
			\t@which tinygo > /dev/null || echo "TinyGo not found. Please install from https://tinygo.org"
			\tgo mod download
			\t@echo "Development environment ready!"

			fmt:
			\tgo fmt ./...

			lint:
			\t@which golangci-lint > /dev/null || (echo "golangci-lint not found. Run 'make dev-setup' first." && exit 1)
			\tgolangci-lint run

			test:
			\tgo test -v ./...

			test-cov:
			\tgo test -v -coverprofile=coverage.out ./...
			\tgo tool cover -html=coverage.out -o coverage.html
			\t@echo "Coverage report generated: coverage.html"

			clean:
			\trm -f *.wasm app.wasm
			\trm -f coverage.out coverage.html
			\tgo clean

			build: clean
			\t@echo "Building WebAssembly module..."
			\t@which tinygo > /dev/null || (echo "TinyGo not found. Please install from https://tinygo.org" && exit 1)
			\ttinygo build -target=wasip1 -gc=leaking -scheduler=none -no-debug -o main.wasm main.go
			\t@echo "Built: main.wasm"

			# Run all quality checks
			quality: fmt lint test
			\t@echo "All quality checks passed!"

			# Verify TinyGo compatibility
			verify-tinygo:
			\t@echo "Checking TinyGo compatibility..."
			\ttinygo build -target=wasip1 -gc=leaking -scheduler=none -no-debug -o /tmp/test.wasm main.go && rm /tmp/test.wasm
			\t@echo "TinyGo build successful!"

			# Quick development cycle
			dev: fmt lint test build
			\t@echo "Development cycle complete!"
			"""
		
		"main.go": """
			package main

			import (
			\t"github.com/fastertools/ftl-cli/sdk/go/ftl"
			)

			// ExampleToolInput defines the input for the example tool
			type ExampleToolInput struct {
			\tMessage string `json:"message" description:"The input message to process"`
			}

			// ExampleTool processes messages
			func ExampleTool(input ExampleToolInput) ftl.ToolResponse {
			\t// TODO: Implement your tool logic here
			\treturn ftl.Text("Processed: " + input.Message)
			}

			func main() {
			\t// Register tools
			\tftl.Handle(ftl.Tools{
			\t\t"exampleTool": {
			\t\t\tDescription: "An example tool that processes messages",
			\t\t\tInputSchema: ExampleToolInput{},
			\t\t\tHandler:     ExampleTool,
			\t\t},
			\t\t
			\t\t// Add more tools here as needed:
			\t\t// "anotherTool": {
			\t\t//     Description: "Another tool description",
			\t\t//     InputSchema: AnotherInput{},
			\t\t//     Handler:     AnotherTool,
			\t\t// },
			\t})
			}
			"""
		
		"main_test.go": """
			package main

			import (
			\t"testing"
			)

			func TestExampleTool(t *testing.T) {
			\tinput := ExampleToolInput{
			\t\tMessage: "Hello, World!",
			\t}
			\t
			\tresponse := ExampleTool(input)
			\t
			\tif response.Text != "Processed: Hello, World!" {
			\t\tt.Errorf("Expected 'Processed: Hello, World!', got '%s'", response.Text)
			\t}
			}
			"""
		
		".gitignore": """
			*.wasm
			*.exe
			*.test
			coverage.out
			coverage.html
			"""
	}
}

// Template selector based on language
#Templates: {
	rust:       #RustComponent
	typescript: #TypeScriptComponent
	python:     #PythonComponent
	go:         #GoComponent
}