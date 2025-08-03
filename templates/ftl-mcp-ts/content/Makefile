.PHONY: build clean test install format lint

install:
	npm install

build: install
	npm run build

clean:
	rm -rf dist node_modules

test: install
	npm test

format: install
	npm run format || echo "No format script defined"

lint: install
	npm run lint || echo "No lint script defined"

dev: install format lint test
	@echo "Development checks passed!"