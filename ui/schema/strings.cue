package ui

// Process display names - CRITICAL for fixing "FTL Process" vs "Up Process" bug
#ProcessNames: {
	regular: "FTL Process"
	watch:   "Watch Mode"
	build:   "Build Process"
	none:    "No Active Process"
}

// Status indicators used across UI
#StatusStrings: {
	stopped:  "Stopped"
	running:  "Running"
	inactive: "Inactive"
	starting: "Starting..."
	stopping: "Stopping..."
	building: "Building..."
	error:    "Error"
}

// Web UI Labels
#WebUI: {
	titles: {
		main:           "FTL Control Center"
		control_center: "Control Center"
		projects:       "Projects"
		activity:       "Activity"
		project_info:   "Project Info"
		tools:          "Available Tools"
	}
	
	tabs: {
		live_logs:      "Live Logs"
		command_output: "Command Output"
	}
	
	buttons: {
		build:       "Build"
		up:          "Up"
		watch:       "Watch"
		stop:        "Stop"
		clear:       "Clear"
		add_project: "+ Add Project"
		cancel:      "Cancel"
		remove:      "Remove"
		parameters:  "Parameters"
	}
	
	messages: {
		waiting_logs:      "Waiting for log output..."
		ready_commands:    "Ready to execute commands..."
		loading_tools:     "Loading tools..."
		no_tools:          "No tools available"
		no_parameters:     "No parameters"
		sending_build:     "Sending build command..."
		starting_server:   "Starting FTL server..."
		starting_watch:    "Starting FTL in watch mode..."
	}
	
	forms: {
		project_name:    "Project name"
		project_path:    "/path/to/project"
		confirm_remove:  "Remove this project?"
	}
}

// CLI Command Strings
#CLI: {
	commands: {
		build: {
			short: "Build the FTL application"
			long:  "Build compiles the FTL application and its components."
		}
		up: {
			short: "Run the FTL application locally"
			long:  "Run the FTL application locally with hot reload support."
		}
		deploy: {
			short: "Deploy the FTL application to the platform"
			long: """
				Deploy the FTL application to the platform.
				
				This command:
				1. Reads your FTL configuration (ftl.yaml, ftl.json, or app.cue)
				2. Builds local components
				3. Creates/updates the app on FTL platform
				4. Pushes built components to the registry
				5. Sends the FTL config to the platform for deployment
				6. Platform synthesizes Spin manifest and deploys
				
				Example:
				  ftl deploy
				  ftl deploy --access-control private
				  ftl deploy --jwt-issuer https://auth.example.com --jwt-audience api.example.com
				  ftl deploy --dry-run
				"""
		}
	}
	
	messages: {
		synthesizing:    "Synthesizing spin.toml from %s"
		generated:       "Generated spin.toml"
		building:        "Building FTL application..."
		build_complete:  "Build completed successfully"
		starting:        "Starting FTL application..."
		watch_mode:      "Starting with watch mode..."
	}
	
	errors: {
		no_config:        "no ftl.yaml, ftl.json, app.cue, or spin.toml found. Run 'ftl init' first"
		no_spin_toml:     "no spin.toml found. Run 'ftl synth' or 'ftl build' without --skip-synth first"
		build_failed:     "Build failed: %s"
		synthesis_failed: "Synthesis failed: %s"
	}
	
	flags: {
		build:      "Build before running"
		watch:      "Watch for changes and reload"
		config:     "Configuration file to synthesize (auto-detects if not specified)"
		skip_synth: "Skip synthesis of spin.toml from FTL config"
		env:        "Pass an environment variable (key=value) to all components of the application"
	}
}

// MCP Server Strings
#MCP: {
	server: {
		name:        "mcp-server"
		description: "FTL server - handles ftl up operations in regular and watch modes"
	}
	
	tools: {
		up: {
			name:        "mcp-server__up"
			description: "Run ftl up in regular or watch mode"
		}
		stop: {
			name:        "mcp-server__stop"
			description: "Stop any running FTL process (watch or regular mode)"
		}
		build: {
			name:        "mcp-server__build"
			description: "Run ftl build command"
		}
		status: {
			name:        "mcp-server__get_status"
			description: "Get current status of FTL processes"
		}
		components: {
			name:        "mcp-server__list_components"
			description: "List all components in the FTL project"
		}
		logs: {
			name:        "mcp-server__get_logs"
			description: "Get logs from running watch process"
		}
	}
	
	messages: {
		already_running:  "FTL process already running in %s mode"
		process_started:  "Started 'ftl up%s%s' in project: %s"
		initial_output:   "Initial output:"
		build_failed:     "Build failed: %s"
		process_stopped:  "FTL Process stopped: %s"
	}
	
	errors: {
		empty_address:   "empty listen address"
		invalid_address: "invalid listen address format: %s"
		port_out_range:  "port %d is out of valid range (1-65535)"
		process_exists:  "Process already exists"
		failed_action:   "Failed to %s: %s"
	}
}

// HTTP Handler Messages
#Handlers: {
	errors: {
		method_not_allowed:     "Method not allowed"
		parse_form_failed:      "Failed to parse form"
		no_projects:            "No projects configured"
		project_path_required:  "Error: project_path required"
		process_type_required:  "Error: project_path and process_type required"
		invalid_path:           "Invalid path"
		invalid_index:          "Invalid index"
		no_current_project:     "No current project"
		no_running_server:      "No running server"
		failed_get_tools:       "Failed to get tools"
		tool_not_found:         "Tool not found"
		name_path_required:     "Name and path are required"
		directory_not_exist:    "Directory does not exist: %s"
		not_ftl_project:        "Not a valid FTL project: %s"
	}
	
	success: {
		projects_reloaded: "Projects reloaded successfully"
		build_successful:  "Build successful: %s"
		build_failed:      "Build failed: %s"
		process_stopped:   "Process stopped successfully"
		stop_failed:       "Stop failed: %s"
		start_failed:      "Start failed: %s"
		watch_start_failed: "Watch start failed: %s"
	}
}