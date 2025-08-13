// Core Spin manifest schemas
package core

import "strings"

// SpinManifest represents a complete Spin application manifest
#SpinManifest: {
	spin_manifest_version: 2
	
	application: #Application
	variables?: [string]: #Variable
	component: [string]: #Component
	trigger: http: [...#HttpTrigger]
}

// Application metadata
#Application: {
	name!: string & =~"^[a-zA-Z][a-zA-Z0-9_-]*$"
	version?: string | *"0.1.0"
	description?: string
	authors?: [...string]
}

// Variable definition
#Variable: {
	default?: string
	required?: bool | *false
}

// Component definition
#Component: {
	source!: string
	allowed_outbound_hosts?: [...string]
	environment?: [string]: string
	variables?: [string]: string
	files?: [...string | #FileMount]
	build?: #BuildConfig
}

// File mount configuration
#FileMount: {
	source!: string
	destination!: string
}

// Build configuration
#BuildConfig: {
	command!: string
	workdir?: string
	watch?: [...string]
}

// HTTP trigger configuration
#HttpTrigger: {
	route!: string
	component!: string
	executor?: string
}

// Source normalization helper
#NormalizeSource: {
	input: string
	
	output: string
	if strings.HasPrefix(input, "ghcr.io/") || strings.HasPrefix(input, "docker.io/") {
		output: input
	}
	if !strings.HasPrefix(input, "ghcr.io/") && !strings.HasPrefix(input, "docker.io/") {
		if strings.HasSuffix(input, ".wasm") {
			output: input
		}
		if !strings.HasSuffix(input, ".wasm") {
			output: input + ".wasm"
		}
	}
}