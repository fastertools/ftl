module github.com/fastertools/ftl-cli/go/shared

go 1.24

require (
	cuelang.org/go v0.14.1
	github.com/aws/aws-lambda-go v1.49.0
	github.com/fastertools/ftl-cli/go/shared/auth v0.0.0
	github.com/google/uuid v1.6.0
	github.com/oapi-codegen/runtime v1.1.2
	github.com/pkg/errors v0.9.1
	github.com/stretchr/testify v1.9.0
	gopkg.in/yaml.v3 v3.0.1
)

require (
	al.essio.dev/pkg/shellescape v1.5.1 // indirect
	github.com/apapsch/go-jsonmerge/v2 v2.0.0 // indirect
	github.com/cockroachdb/apd/v3 v3.2.1 // indirect
	github.com/danieljoos/wincred v1.2.2 // indirect
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/emicklei/proto v1.14.2 // indirect
	github.com/godbus/dbus/v5 v5.1.0 // indirect
	github.com/mitchellh/go-wordwrap v1.0.1 // indirect
	github.com/pelletier/go-toml/v2 v2.2.4 // indirect
	github.com/pkg/browser v0.0.0-20240102092130-5ac0b6a4141c // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/protocolbuffers/txtpbfmt v0.0.0-20250627152318-f293424e46b5 // indirect
	github.com/zalando/go-keyring v0.2.6 // indirect
	golang.org/x/net v0.42.0 // indirect
	golang.org/x/sys v0.34.0 // indirect
	golang.org/x/text v0.27.0 // indirect
)

replace github.com/fastertools/ftl-cli/go/shared/auth => ./auth
