# CLI Reference

The `ftl` command-line interface is the primary entry point for developers using the FTL platform. It provides a number of commands for creating, testing, and deploying tools.

## Global Options

- `-v, --verbose`: Increase logging verbosity.

## `ftl new`

Create a new tool from a template.

```bash
ftl new <name> [OPTIONS]
```

### Arguments

- `<name>`: The name of the tool.

### Options

- `-d, --description <description>`: The description of the tool.

## `ftl build`

Build a tool.

```bash
ftl build [name] [OPTIONS]
```

### Arguments

- `[name]`: The name of the tool to build (defaults to the current directory).

### Options

- `-p, --profile <profile>`: The build profile to use (`dev`, `release`, or `tiny`).
- `-s, --serve`: Start a local development server after the build completes.

## `ftl serve`

Serve a tool locally.

```bash
ftl serve [name] [OPTIONS]
```

### Arguments

- `[name]`: The name of the tool to serve (defaults to the current directory).

### Options

- `-p, --port <port>`: The port to serve on (defaults to 3000).
- `-b, --build`: Build the tool before serving.

## `ftl test`

Run tests for a tool.

```bash
ftl test [name]
```

### Arguments

- `[name]`: The name of the tool to test (defaults to the current directory).

## `ftl deploy`

Deploy a tool to the FTL Edge.

```bash
ftl deploy [name]
```

### Arguments

- `[name]`: The name of the tool to deploy (defaults to the current directory).

## `ftl toolkit`

Manage toolkits (collections of tools).

### `ftl toolkit build`

Build a toolkit from multiple tools.

```bash
ftl toolkit build --name <name> <tools...>
```

#### Options

- `--name <name>`: The name of the toolkit.

#### Arguments

- `<tools...>`: The tools to include in the toolkit.

### `ftl toolkit serve`

Serve a toolkit locally.

```bash
ftl toolkit serve <name> [OPTIONS]
```

#### Arguments

- `<name>`: The name of the toolkit.

#### Options

- `-p, --port <port>`: The port to serve on (defaults to 3000).

### `ftl toolkit deploy`

Deploy a toolkit to the FTL Edge.

```bash
ftl toolkit deploy <name>
```

#### Arguments

- `<name>`: The name of the toolkit.
