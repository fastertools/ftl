# CLI Reference

The `ftl` command-line interface provides commands for creating, building, testing, and deploying MCP components.

## Global Options

- `-v, --verbose`: Increase logging verbosity (can be used multiple times)
- `--help`: Show help information
- `--version`: Show version information

## Project & Component Commands

### `ftl init`

Create a new MCP project for composing components.

```bash
ftl init [name] [OPTIONS]
```

**Arguments:**
- `[name]`: Project name (optional, will prompt if not provided)

**Options:**
- `--here`: Initialize in current directory

**Examples:**
```bash
# Create a new project
ftl init my-assistant

# Initialize in current directory
ftl init my-project --here

# Interactive mode
ftl init
```

### `ftl add`

Add a new MCP component to the current project.

```bash
ftl add [name] [OPTIONS]
```

**Arguments:**
- `[name]`: Component name (optional, will prompt if not provided)

**Options:**
- `-l, --language <lang>`: Language to use (`rust`, `typescript`, `javascript`)
- `-d, --description <desc>`: Component description
- `-r, --route <route>`: HTTP route for the component (default: `/[name]`)
- `--git <url>`: Use a Git repository as the template source
- `--branch <name>`: Git branch to use (requires `--git`)
- `--dir <path>`: Use a local directory as the template source
- `--tar <path>`: Use a tarball as the template source

**Examples:**
```bash
# Add a component with specified language
ftl add weather-api --language typescript --description "Weather data for AI agents"

# Add a component with custom route
ftl add calculator --language rust --route /calc

# Using a custom Git template
ftl add my-component --git https://github.com/user/template --branch main

# Interactive mode
ftl add
```

### `ftl build`

Build the component or project in the current directory.

```bash
ftl build [OPTIONS]
```

**Options:**
- `-r, --release`: Build in release mode
- `-p, --path <path>`: Path to component or project directory (default: current)

**Examples:**
```bash
# Build entire project (from project root with spin.toml)
ftl build

# Build specific component
cd math-tools
ftl build --release

# Build specific component from project root
ftl build --path math-tools
```

### `ftl up`

Run the component locally for development.

```bash
ftl up [OPTIONS]
```

**Options:**
- `--build`: Build before running
- `-p, --port <port>`: Port to serve on (default: 3000)
- `--path <path>`: Path to component directory

**Example:**
```bash
ftl up --build --port 8080
```

### `ftl test`

Run component tests.

```bash
ftl test [OPTIONS]
```

**Options:**
- `-p, --path <path>`: Path to component directory

**Example:**
```bash
ftl test
```

### `ftl publish`

Publish component to an OCI registry.

```bash
ftl publish [OPTIONS]
```

**Options:**
- `-r, --registry <url>`: Registry URL (default: ghcr.io)
- `-t, --tag <version>`: Version tag to publish
- `--path <path>`: Path to component directory

**Example:**
```bash
ftl publish --tag v1.0.0
ftl publish --registry docker.io --tag latest
```



### `ftl deploy`

Deploy the project to FTL.

```bash
ftl deploy [OPTIONS]
```

**Options:**
- `-e, --environment <name>`: Target environment

**Example:**
```bash
ftl deploy --environment production
```

## Configuration Commands

### `ftl setup templates`

Install or update FTL component templates.

```bash
ftl setup templates [OPTIONS]
```

**Options:**
- `--force`: Force reinstall even if already installed
- `--git <url>`: Install templates from a Git repository
- `--branch <name>`: Git branch to use (requires `--git`)
- `--dir <path>`: Install templates from a local directory
- `--tar <path>`: Install templates from a tarball

**Examples:**
```bash
# Install default FTL templates
ftl setup templates

# Install templates from a Git repository
ftl setup templates --git https://github.com/user/ftl-templates --branch main

# Install templates from a local directory
ftl setup templates --dir ./my-templates

# Install templates from a tarball
ftl setup templates --tar ./templates.tar.gz

# Force reinstall templates
ftl setup templates --force
```

### `ftl setup info`

Show FTL configuration and status.

```bash
ftl setup info
```

Displays:
- FTL CLI version
- Spin installation status
- Template installation status
- wkg availability

## Registry Commands

### `ftl registry list`

List available components (coming soon).

```bash
ftl registry list [OPTIONS]
```

**Options:**
- `-r, --registry <url>`: Registry to list from

### `ftl registry search`

Search for components (coming soon).

```bash
ftl registry search <query> [OPTIONS]
```

**Arguments:**
- `<query>`: Search query

**Options:**
- `-r, --registry <url>`: Registry to search in

### `ftl registry info`

Show component details (coming soon).

```bash
ftl registry info <component>
```

**Arguments:**
- `<component>`: Component name or URL

## Environment Variables

- `FTL_AUTO_INSTALL`: Set to `true` to auto-install Spin without prompting
- `RUST_LOG`: Control logging verbosity (e.g., `info`, `debug`, `trace`)

## Exit Codes

- `0`: Success
- `1`: General error
- `2`: Invalid arguments
- `127`: Command not found