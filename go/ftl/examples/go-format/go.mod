module demo-app

go 1.24

toolchain go1.24.6

require github.com/fastertools/ftl-cli/go/ftl v0.1.0

require (
	cuelang.org/go v0.10.1 // indirect
	github.com/cockroachdb/apd/v3 v3.2.1 // indirect
	github.com/google/uuid v1.6.0 // indirect
	golang.org/x/mod v0.20.0 // indirect
	golang.org/x/net v0.28.0 // indirect
	golang.org/x/text v0.17.0 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

// For local development, uncomment and adjust the path:
replace github.com/fastertools/ftl-cli/go/ftl => /home/ian/Dev/ftl-cli/go/ftl
