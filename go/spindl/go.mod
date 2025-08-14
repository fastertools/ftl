module github.com/fastertools/ftl-cli/go/spindl

go 1.24

require (
	cuelang.org/go v0.10.1
	github.com/BurntSushi/toml v1.3.2
	github.com/fastertools/ftl-cli/go/shared v0.0.0
	github.com/fatih/color v1.18.0
	github.com/spf13/cobra v1.8.1
	gopkg.in/yaml.v3 v3.0.1
)

require (
	github.com/cockroachdb/apd/v3 v3.2.1 // indirect
	github.com/google/uuid v1.6.0 // indirect
	github.com/inconshreveable/mousetrap v1.1.0 // indirect
	github.com/mattn/go-colorable v0.1.13 // indirect
	github.com/mattn/go-isatty v0.0.20 // indirect
	github.com/spf13/pflag v1.0.5 // indirect
	golang.org/x/mod v0.20.0 // indirect
	golang.org/x/net v0.28.0 // indirect
	golang.org/x/sys v0.25.0 // indirect
	golang.org/x/text v0.17.0 // indirect
)

replace github.com/fastertools/ftl-cli/go/shared => ../shared
