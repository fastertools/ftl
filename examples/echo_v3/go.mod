module github.com/fastertools/ftl-cli/examples/echo_v3

go 1.23

require github.com/fastertools/ftl-cli/sdk/go v0.0.0

replace github.com/fastertools/ftl-cli/sdk/go => ../../sdk/go

require (
	github.com/julienschmidt/httprouter v1.3.0 // indirect
	github.com/spinframework/spin-go-sdk v0.0.0-20250411015808-ee0bd1e7d170 // indirect
)
