package ftl_mcp

import (
	"list"
)

// ===========================================================================
// MCP Testing Schema Definitions
// ===========================================================================

// Core test configuration that defines what we're testing
#TestConfig: {
	name!:        string & =~"^[a-z][a-z0-9-_]*$"
	description:  string | *""
	test_type:    "unit" | "integration" | "e2e" | *"integration"
	timeout_ms:   number & >=1000 | *30000  // Default 30 second timeout
	
	// Test environment setup
	environment?: {
		test_mode:     bool | *true
		ftl_path?:     string  // Override FTL binary path for testing
		project_path?: string  // Test project location
		cleanup:       bool | *true  // Auto-cleanup after test
	}
	
	// Dependencies and prerequisites
	requires?: {
		tools?: [...string]          // Required FTL tools
		files?: [...string]          // Required files
		ports?: [...number]          // Required available ports
		env_vars?: {[string]: string}  // Required environment variables
	}
}

// MCP tool test definition - validates tool inputs/outputs
#MCPToolTest: {
	#TestConfig
	
	// Tool being tested
	tool_name!: string & =~"^mcp-server__[a-z][a-z0-9-_]*$"
	
	// Test scenarios for this tool
	scenarios!: [...#TestScenario] & list.MinItems(1)
}

// Individual test scenario within a tool test
#TestScenario: {
	name!:        string
	description?: string
	
	// Input to send to MCP tool
	input!: {
		// Common input fields that all tools might have
		project_path?: string
		
		// Tool-specific inputs (validated against tool's input schema)
		...
	}
	
	// Expected output validation
	expect!: {
		success:  bool | *true
		
		// Expected response structure
		response?: {
			// Files that should be created/modified
			files_created?: [...string]
			files_modified?: [...string]
			
			// Output messages/logs
			messages?: [...string]
			
			// JSON response validation
			json?: {...}
		}
		
		// File system state validation
		filesystem?: {
			files_exist?: [...string]
			files_not_exist?: [...string]
			file_contents?: {[string]: string}  // file_path: expected_content
		}
		
		// Process state validation
		processes?: {
			running?: [...string]    // Process names that should be running
			not_running?: [...string]  // Process names that should not be running
		}
	}
	
	// Setup required before running this scenario
	setup?: #TestSetup
	
	// Cleanup after scenario (in addition to environment.cleanup)
	cleanup?: #TestCleanup
}

// Test setup operations
#TestSetup: {
	// Create temporary directories
	create_dirs?: [...string]
	
	// Create temporary files with content
	create_files?: {[string]: string}  // file_path: content
	
	// Environment variables to set
	env_vars?: {[string]: string}
	
	// Commands to run before test
	commands?: [...string]
	
	// Projects to initialize
	init_projects?: [...{
		name: string
		language?: "rust" | "python" | "go" | *"rust"
		path?: string
	}]
}

// Test cleanup operations  
#TestCleanup: {
	// Directories to remove
	remove_dirs?: [...string]
	
	// Files to remove
	remove_files?: [...string]
	
	// Processes to kill
	kill_processes?: [...string]
	
	// Ports to free up
	free_ports?: [...number]
	
	// Commands to run for cleanup
	commands?: [...string]
}

// ===========================================================================
// E2E Test Suite Configuration
// ===========================================================================

// Full end-to-end test suite
#E2ETestSuite: {
	#TestConfig
	test_type: "e2e"
	
	// Test flow - ordered sequence of operations
	flow!: [...#E2EStep] & list.MinItems(1)
	
	// Global setup for entire suite
	suite_setup?: #TestSetup
	
	// Global cleanup for entire suite
	suite_cleanup?: #TestCleanup
}

// Individual step in E2E test flow
#E2EStep: {
	name!: string
	type!: "mcp_call" | "command" | "verify" | "wait"
	
	// For mcp_call steps
	if type == "mcp_call" {
		tool_name!: string
		input!: {...}
		expect?: #TestScenario.expect
	}
	
	// For command steps
	if type == "command" {
		command!: string
		args?: [...string]
		expect_success?: bool | *true
		expect_output?: string
	}
	
	// For verify steps
	if type == "verify" {
		filesystem?: {
			files_exist?: [...string]
			files_not_exist?: [...string]
			file_contents?: {[string]: string}  // file_path: expected_content
		}
		processes?: {
			running?: [...string]    // Process names that should be running
			not_running?: [...string]  // Process names that should not be running
		}
		custom?: {...}  // Custom verification logic
	}
	
	// For wait steps
	if type == "wait" {
		duration_ms?: number & >=100 | *1000
		for_condition?: {
			type: "file_exists" | "process_running" | "port_available"
			target: string
			timeout_ms?: number & >=100 | *5000
		}
	}
}

// ===========================================================================
// Project Lifecycle Test Templates
// ===========================================================================

// Standard project lifecycle test (init -> add -> up -> stop)
#ProjectLifecycleTest: #E2ETestSuite & {
	name: "project_lifecycle"
	description: "Test complete project lifecycle from init to cleanup"
	
	// Standard flow for project testing
	flow: [
		{
			name: "init_project"
			type: "mcp_call"
			tool_name: "mcp-server__init"
			input: {
				project_name: "test-project"
				language: "rust"
				base_path: "./test-projects"
			}
			expect: {
				success: true
				filesystem: {
					files_exist: [
						"./test-projects/test-project/ftl.toml",
						"./test-projects/test-project/Cargo.toml"
					]
				}
			}
		},
		{
			name: "add_tool"
			type: "mcp_call"
			tool_name: "mcp-server__add"
			input: {
				tool_name: "hello-world"
				language: "rust"
				project_path: "./test-projects/test-project"
			}
			expect: {
				success: true
				filesystem: {
					files_exist: [
						"./test-projects/test-project/tools/hello-world/Cargo.toml",
						"./test-projects/test-project/tools/hello-world/src/lib.rs"
					]
				}
			}
		},
		{
			name: "build_project"
			type: "mcp_call"
			tool_name: "mcp-server__build"
			input: {
				project_path: "./test-projects/test-project"
			}
			expect: {
				success: true
			}
		},
		{
			name: "start_server"
			type: "mcp_call"
			tool_name: "mcp-server__up"
			input: {
				project_path: "./test-projects/test-project"
				port: 8090
			}
			expect: {
				success: true
				processes: {
					running: ["ftl"]
				}
			}
		},
		{
			name: "verify_server_running"
			type: "verify"
			processes: {
				running: ["ftl"]
			}
		},
		{
			name: "stop_server"
			type: "mcp_call"
			tool_name: "mcp-server__stop"
			input: {
				project_path: "./test-projects/test-project"
			}
			expect: {
				success: true
				processes: {
					not_running: ["ftl"]
				}
			}
		}
	]
	
	suite_setup: {
		create_dirs: ["./test-projects"]
		env_vars: {
			"FTL_TEST_MODE": "true"
		}
	}
	
	suite_cleanup: {
		remove_dirs: ["./test-projects"]
		kill_processes: ["ftl"]
		free_ports: [8090]
	}
}

// ===========================================================================
// Validation Schemas for Common Tool Inputs
// ===========================================================================

// Init tool input validation
#InitToolInput: {
	project_name!: string & =~"^[a-z][a-z0-9-]*$"
	language:      "rust" | "python" | "go" | *"rust"
	base_path:     string | *"."
	template?:     string
}

// Add tool input validation
#AddToolInput: {
	tool_name!:    string & =~"^[a-z][a-z0-9-]*$"
	language:      "rust" | "python" | "go" | *"rust"
	project_path!: string
	template?:     string
}

// Build tool input validation
#BuildToolInput: {
	project_path!: string
	target?:       string
	release?:      bool | *false
}

// Up tool input validation  
#UpToolInput: {
	project_path!: string
	port?:         number & >=1024 & <=65535 | *8080
	detach?:       bool | *false
}

// Stop tool input validation
#StopToolInput: {
	project_path!: string
	force?:        bool | *false
}

// Status tool input validation
#StatusToolInput: {
	project_path!: string
	detailed?:     bool | *false
}

// Logs tool input validation
#LogsToolInput: {
	project_path!: string
	lines?:        number & >=1 | *100
	follow?:       bool | *false
}

// ===========================================================================
// Test-Specific MCP Tools Input Validation
// ===========================================================================

// Test server management tools (mcp-server/tools/testing/)
#ServerStartInput: {
	use_test_projects?: bool | *true
	project_base?:      string | *"./e2e-projects"
	timeout_ms?:        number & >=1000 | *30000
	ftl_binary_path?:   string | *"ftl"
	port?:              number & >=1024 & <=65535 | *8080
}

#ServerStopInput: {
	graceful?: bool | *true
	timeout_ms?: number & >=1000 | *5000
}

#ServerStatusInput: {
	detailed?: bool | *false
}

// Test project management
#GenerateTestProjectNameInput: {
	prefix?: string | *"test"
	suffix?: string | *""
}

#EnsureTestDirectoryInput: {
	base_path?: string | *"./e2e-projects"
	project_name?: string
}

#CleanupTestProjectsInput: {
	remove_directories?: bool | *true
	base_path?:          string | *"./e2e-projects"
	max_age_minutes?:    number | *60  // Clean projects older than 1 hour
}

// Test utilities
#WaitReadyInput: {
	url!:           string
	timeout_ms?:    number & >=1000 | *10000
	interval_ms?:   number & >=100 | *500
	expected_status?: number | *200
}

#HealthCheckInput: {
	project_path?: string
	check_processes?: bool | *true
	check_ports?:     bool | *true
	check_files?:     bool | *false
}

#ProcessInfoInput: {
	project_path?: string
	include_children?: bool | *true
}

// ===========================================================================
// Test Registry and Discovery
// ===========================================================================

// Test registry for organizing and discovering tests
#TestRegistry: {
	// Metadata
	version: string | *"1.0.0"
	
	// Test suites organized by category
	unit_tests: [...#MCPToolTest]
	integration_tests: [...#MCPToolTest] 
	e2e_tests: [...#E2ETestSuite]
	
	// Predefined test templates
	templates: {
		project_lifecycle: #ProjectLifecycleTest
		// More templates can be added here
	}
	
	// Test configuration defaults
	defaults: #TestConfig & {
		timeout_ms: 30000
		environment: {
			test_mode: true
			cleanup: true
		}
	}
}

// ===========================================================================
// TypeScript Integration Schema Mappings
// ===========================================================================

// Maps MCP tool names to their input validation schemas
// Used by TypeScript CUE validation wrapper
#ToolSchemaMapping: {
	// Standard FTL tools
	"mcp-server__init":   #InitToolInput
	"mcp-server__add":    #AddToolInput  
	"mcp-server__build":  #BuildToolInput
	"mcp-server__up":     #UpToolInput
	"mcp-server__stop":   #StopToolInput
	"mcp-server__status": #StatusToolInput
	"mcp-server__logs":   #LogsToolInput
	
	// Test-specific tools
	"mcp-server__server_start":               #ServerStartInput
	"mcp-server__server_stop":                #ServerStopInput
	"mcp-server__server_status":              #ServerStatusInput
	"mcp-server__generate_test_project_name": #GenerateTestProjectNameInput
	"mcp-server__ensure_test_directory":      #EnsureTestDirectoryInput
	"mcp-server__cleanup_test_projects":      #CleanupTestProjectsInput
	"mcp-server__wait_ready":                 #WaitReadyInput
	"mcp-server__health_check":               #HealthCheckInput
	"mcp-server__process_info":               #ProcessInfoInput
}

// JSON Schema export helper for TypeScript integration
// Allows generating JSON Schema from CUE for runtime validation
#JSONSchemaExport: {
	tools: #ToolSchemaMapping
	common: {
		test_config: #TestConfig
		test_scenario: #TestScenario
		test_setup: #TestSetup
		test_cleanup: #TestCleanup
	}
}