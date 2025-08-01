module github.com/example/{{project-name | kebab_case}}

go 1.21

// Version: v0.1.0

require (
	github.com/fastertools/ftl-cli/sdk/go v0.0.0
)

// Use local FTL SDK during development
replace github.com/fastertools/ftl-cli/sdk/go => ${FTL_SDK_PATH}/sdk/go