.PHONY: build test clean install

install:
	cd handler && npm install

build: install
	cd handler && npm run build

test: install
	cd handler && npm test

clean:
	cd handler && npm run clean
	cd handler && rm -rf node_modules

serve:
	ftl serve

deploy:
	ftl deploy