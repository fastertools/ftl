.PHONY: help install install-dev format lint type-check test test-cov clean build

help:
	@echo "Available commands:"
	@echo "  install      Install project dependencies"
	@echo "  install-dev  Install project with development dependencies"
	@echo "  format       Format code with black"
	@echo "  lint         Run linting with ruff"
	@echo "  type-check   Run type checking with mypy"
	@echo "  test         Run tests"
	@echo "  test-cov     Run tests with coverage"
	@echo "  clean        Clean build artifacts"
	@echo "  build        Build WebAssembly module"

install:
	pip install -e .

install-dev:
	pip install -e ".[dev]"
	@echo "Installing componentize-py for WebAssembly builds..."
	pip install componentize-py

format:
	black src tests

lint:
	ruff check src tests --fix

type-check:
	mypy src

test:
	pytest

test-cov:
	pytest --cov=src --cov-report=term-missing --cov-report=html

clean:
	rm -rf build dist *.egg-info
	rm -rf .coverage htmlcov .pytest_cache .mypy_cache .ruff_cache
	rm -f app.wasm
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete

build: clean
	@echo "Building WebAssembly module..."
	@which componentize-py > /dev/null || (echo "componentize-py not found. Run 'make install-dev' first." && exit 1)
	componentize-py -w spin-http componentize src/main.py -o app.wasm

# Development workflow
dev: install-dev
	@echo "Development environment ready!"
	@echo "Run 'make test' to run tests"
	@echo "Run 'make format' to format code"
	@echo "Run 'make lint' to run linting"
	@echo "Run 'make build' to build WebAssembly module"