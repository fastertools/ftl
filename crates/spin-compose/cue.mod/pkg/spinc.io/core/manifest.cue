// Core Spin manifest schema (L1 constructs)
package core

// Spin manifest root structure
#SpinManifest: {
    spin_manifest_version: 2
    
    application: #Application
    
    variables?: [string]: #Variable
    
    component?: [string]: #Component
    
    trigger?: #Triggers
}

// Application metadata
#Application: {
    name!: string & =~"^[a-zA-Z][a-zA-Z0-9_-]*$"
    version?: string | *"0.1.0"
    description?: string
    authors?: [...string]
    
    trigger?: {
        redis?: {
            address: string
        }
    }
}

// Variable types
#Variable: {
    default: string
} | {
    required: true
} | {
    default: string
    secret: true
} | {
    required: true
    secret: true
}

// Component definition
#Component: {
    description?: string
    
    source!: string | #RegistrySource | #UrlSource
    
    files?: [...string | #FileMount]
    exclude_files?: [...string]
    
    allowed_outbound_hosts?: [...string]
    allowed_http_hosts?: [...string]  // Legacy, prefer allowed_outbound_hosts
    
    key_value_stores?: [...string]
    
    environment?: [string]: string
    
    build?: #BuildConfig
    
    variables?: [string]: string
    
    dependencies_inherit_configuration?: bool | *false
    
    dependencies?: [string]: #ComponentDependency
}

// Registry source for components
#RegistrySource: {
    registry!: string
    package!: string
    version!: string
}

// URL source for components
#UrlSource: {
    url!: string
    digest!: string & =~"^sha256:[a-f0-9]{64}$"
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

// Component dependency
#ComponentDependency: {
    version?: string
} | #RegistrySource

// Trigger definitions
#Triggers: {
    http?: [...#HttpTrigger]
    redis?: [...#RedisTrigger]
}

// HTTP trigger
#HttpTrigger: {
    route!: string | { private: true }
    component!: string | #Component
    executor?: #Executor
}

// Redis trigger
#RedisTrigger: {
    address?: string
    channel!: string
    component!: string | #Component
}

// Executor types
#Executor: {
    type: "spin"
} | {
    type: "wagi"
    argv?: string
    entrypoint?: string
}