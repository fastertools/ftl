module github.com/fastertools/ftl/examples/demo/weather-go

go 1.24

require github.com/fastertools/ftl/sdk/go v0.1.1

replace github.com/fastertools/ftl/sdk/go => ../../../sdk/go

require (
	github.com/julienschmidt/httprouter v1.3.0 // indirect
	github.com/spinframework/spin-go-sdk v0.0.0-20250411015808-ee0bd1e7d170 // indirect
)
