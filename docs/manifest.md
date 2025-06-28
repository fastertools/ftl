# The ftl.toml Manifest

The `ftl.toml` file is the manifest for your FTL tool. It contains metadata about your tool, as well as configuration for the build process and the runtime environment.

## Example

```toml
[tool]
name = "my-tool"
version = "0.1.0"
description = "A new tool for my agent"

[build]
profile = "release"
features = []

[optimization]
flags = ["-O4"]

[runtime]
allowed_hosts = []
```

## `[tool]`

The `[tool]` section contains metadata about your tool.

- `name`: The name of your tool. This must be unique within your FTL account.
- `version`: The version of your tool. This should follow the [SemVer](https://semver.org/) specification.
- `description`: A short description of your tool.

## `[build]`

The `[build]` section contains configuration for the build process.

- `profile`: The build profile to use. This can be `dev`, `release`, or `tiny`.
- `features`: A list of features to enable when building your tool.

## `[optimization]`

The `[optimization]` section contains configuration for the `wasm-opt` tool, which is used to optimize the size and performance of your tool's WebAssembly binary.

- `flags`: A list of flags to pass to `wasm-opt`.

## `[runtime]`

The `[runtime]` section contains configuration for the runtime environment in which your tool will be executed.

- `allowed_hosts`: A list of hosts that your tool is allowed to make HTTP requests to. If this list is empty, your tool will not be able to make any external HTTP requests.
