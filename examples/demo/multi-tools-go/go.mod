module github.com/fastertools/ftl-cli/examples/demo/multi-tools-go

go 1.21

require github.com/fastertools/ftl-cli/sdk/go v0.0.0

replace github.com/fastertools/ftl-cli/sdk/go => ../../../sdk/go

require github.com/fermyon/spin/sdk/go/v2 v2.2.0 // indirect

require github.com/julienschmidt/httprouter v1.3.0 // indirect
