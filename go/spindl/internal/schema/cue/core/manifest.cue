// Core Spin manifest schemas - Spin v3 format
// Based on https://developer.fermyon.com/spin/v3/manifest-reference
package core

// SpinManifest represents a complete Spin application manifest
#SpinManifest: {
	spin_manifest_version: 2
	
	application: #Application
	variables?: [string]: #Variable
	component?: [string]: #Component
	trigger?: #Trigger
}

// Application metadata
#Application: {
	name!: string & =~"^[a-zA-Z0-9][a-zA-Z0-9_-]*$"
	version?: string & =~"^[0-9]+\\.[0-9]+\\.[0-9]+$"
	description?: string
	authors?: [...string]
	trigger?: #ApplicationTrigger
}

// Application-level trigger settings
#ApplicationTrigger: {
	redis?: #ApplicationRedis
}

// Application-level Redis settings
#ApplicationRedis: {
	address!: string & =~"^redis://.*"
}

// Variable definition at application level
#Variable: {
	default?: string
	required?: bool | *false
	secret?: bool | *false
} | {
	default!: string
	required?: bool | *false
	secret?: bool | *false
} | {
	required: true
	secret?: bool | *false
}

// Trigger definitions
#Trigger: {
	http?: [...#HttpTrigger]
	redis?: [...#RedisTrigger]
}

// HTTP trigger configuration
#HttpTrigger: {
	// The component can be either a string reference or an inline component
	component!: string | #Component
	
	// Route can be a string or a table for private routes
	route!: string | #PrivateRoute
	
	// Optional executor configuration
	executor?: #Executor
}

// Private route configuration for service chaining
#PrivateRoute: {
	private: true
}

// Executor configuration
#Executor: {
	type!: "spin" | "wagi"
	
	// Additional fields for WAGI executor
	if type == "wagi" {
		argv?: string
		entrypoint?: string
	}
}

// Redis trigger configuration
#RedisTrigger: {
	// The component can be either a string reference or an inline component
	component!: string | #Component
	
	channel!: string
	address?: string & =~"^redis://.*"
}

// Component definition
#Component: {
	description?: string
	
	// Source can be string (file path) or table (URL/registry)
	source!: string | #URLSource | #RegistrySource
	
	// Files can be strings (paths/globs) or file mount tables
	files?: [...(string | #FileMount)]
	exclude_files?: [...string]
	
	// Network permissions
	allowed_http_hosts?: [...string]  // Deprecated, use allowed_outbound_hosts
	allowed_outbound_hosts?: [...string]
	
	// Storage access
	key_value_stores?: [...string]
	sqlite_databases?: [...string]
	
	// Environment and variables
	environment?: [string]: string
	variables?: [string]: string
	
	// Build configuration
	build?: #BuildConfig
	
	// Component dependencies
	dependencies_inherit_configuration?: bool | *false
	dependencies?: [string]: #Dependency
	
	// AI models (if using AI features)
	ai_models?: [...string]
}

// URL source for downloading components
#URLSource: {
	url!: string & =~"^https?://.*"
	digest!: string & =~"^sha256:[a-fA-F0-9]{64}$"
}

// Registry source for components
#RegistrySource: {
	registry!: string
	package!: string
	version!: string
}

// File mount configuration
#FileMount: {
	source!: string
	destination!: string & =~"^/.*"  // Must be absolute path
}

// Build configuration
#BuildConfig: {
	command!: string
	workdir?: string
	watch?: [...string]
}

// Component dependency specification
#Dependency: #RegistrySource

// Helper type for validating allowed outbound hosts
#AllowedHost: string & =~"^[a-z]+://.*"

// Expression support for certain fields
// These fields support template expressions with application variables
#Expression: string & =~".*\\{\\{.*\\}\\}.*"

// Fields that support expressions:
// - application.trigger.redis.address
// - trigger.redis.address  
// - trigger.redis.channel
// - component.*.allowed_outbound_hosts