.PHONY: help install install-dev format lint type-check test test-cov clean publish

help:
	@echo "Available commands:"
	@echo "  install      Install SDK"
	@echo "  install-dev  Install SDK with development dependencies"
	@echo "  format       Format code with black"
	@echo "  lint         Run linting with ruff"
	@echo "  type-check   Run type checking with mypy"
	@echo "  test         Run tests"
	@echo "  test-cov     Run tests with coverage"
	@echo "  clean        Clean build artifacts"
	@echo "  publish      Build and publish to PyPI (requires credentials)"

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
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete

# Development workflow
dev: install-dev
	@echo "Development environment ready!"
	@echo "Run 'make test' to run tests"
	@echo "Run 'make format' to format code"
	@echo "Run 'make lint' to run linting"
	@echo "Run 'make type-check' to check types"

# Quality checks
quality: format lint type-check test
	@echo "All quality checks passed!"

# Build distribution packages
build: clean
	python -m build

# Publish to PyPI (requires credentials)
publish: build
	python -m twine upload dist/*