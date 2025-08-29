package tools

import (
	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/internal/types"
	"github.com/fastertools/ftl/mcp-server/internal/files"
	"github.com/fastertools/ftl/mcp-server/internal/ftl"
	"github.com/fastertools/ftl/mcp-server/internal/port"
	"github.com/fastertools/ftl/mcp-server/internal/process"
	"github.com/fastertools/ftl/mcp-server/tools/lifecycle"
	"github.com/fastertools/ftl/mcp-server/tools/operations"
	processtools "github.com/fastertools/ftl/mcp-server/tools/process"
	"github.com/fastertools/ftl/mcp-server/tools/project"
	"github.com/fastertools/ftl/mcp-server/tools/testing"
)

// Registry manages all MCP tool registrations
type Registry struct {
	server         *mcp.Server
	fileManager    *files.Manager
	processManager *process.Manager
	portManager    *port.Manager
	ftlCommander   *ftl.Commander
	testMode       bool
	toolConfig     *types.ToolConfig // Universal tool configuration
}

// NewRegistry creates a new tool registry with default configuration
func NewRegistry(server *mcp.Server) *Registry {
	return NewRegistryWithConfig(server, &types.ToolConfig{})
}

// NewRegistryWithConfig creates a new tool registry with custom tool configuration
func NewRegistryWithConfig(server *mcp.Server, config *types.ToolConfig) *Registry {
	// Initialize shared dependencies
	fileManager := files.NewManager()
	processManager := process.NewManager(fileManager)
	portManager := port.NewManager(3000)
	ftlCommander := ftl.NewCommanderWithPath(config.GetFTLPath())

	return &Registry{
		server:         server,
		fileManager:    fileManager,
		processManager: processManager,
		portManager:    portManager,
		ftlCommander:   ftlCommander,
		testMode:       false,
		toolConfig:     config,
	}
}

// EnableTestMode enables registration of test tools
func (r *Registry) EnableTestMode() {
	r.testMode = true
}

// RegisterAll registers all MCP tools with the server
func (r *Registry) RegisterAll() {
	// Create handlers
	upHandler := lifecycle.NewUpHandler(
		r.fileManager,
		r.processManager,
		r.portManager,
		r.ftlCommander,
	)

	stopHandler := lifecycle.NewStopHandler(
		r.fileManager,
		r.processManager,
	)

	statusHandler := lifecycle.NewStatusHandler(
		r.fileManager,
		r.processManager,
	)

	buildHandler := operations.NewBuildHandler(r.ftlCommander)
	
	componentsHandler := operations.NewComponentsHandler(r.ftlCommander)
	
	logsHandler := operations.NewLogsHandler(
		r.fileManager,
		r.processManager,
	)

	// Register lifecycle tools
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__up",
		Description: "Run ftl up in regular or watch mode",
	}, upHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__stop",
		Description: "Stop any running FTL process (watch or regular mode)",
	}, stopHandler.HandleStop)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__get_status",
		Description: "Get current status of FTL processes",
	}, statusHandler.Handle)

	// Register operations tools
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__build",
		Description: "Run ftl build command",
	}, buildHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__list_components",
		Description: "List all components in the FTL project",
	}, componentsHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__get_logs",
		Description: "Get logs from running watch process",
	}, logsHandler.Handle)
	
	// Register project management tools (standard tools)
	r.registerProjectTools()
	
	// Register process management tools
	r.registerProcessTools()
	
	// Register test tools if in test mode
	if r.testMode {
		r.registerTestTools()
	}
}

// registerProjectTools registers project management tools (init, add, etc.)
func (r *Registry) registerProjectTools() {
	// Create project tool handlers
	initHandler := project.NewInitHandler(r.ftlCommander)
	addHandler := project.NewAddHandler(r.ftlCommander)
	
	// Register project management tools
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__init",
		Description: "Initialize a new FTL project",
	}, initHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__add",
		Description: "Add a tool to an FTL project",
	}, addHandler.Handle)
}

// registerProcessTools registers cross-platform process management tools
func (r *Registry) registerProcessTools() {
	// Create process tool handlers
	killGracefullyHandler := processtools.NewKillGracefullyHandler()
	cleanupOrphansHandler := processtools.NewCleanupOrphansHandler()
	verifyStoppedHandler := processtools.NewVerifyStoppedHandler()
	
	// Register process management tools
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__kill_gracefully",
		Description: "Kill process gracefully with SIGTERM, then SIGKILL if needed",
	}, killGracefullyHandler.Handle)
	
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__cleanup_orphans",
		Description: "Find and optionally kill orphaned processes",
	}, cleanupOrphansHandler.Handle)
	
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__verify_stopped",
		Description: "Verify that a process has actually stopped",
	}, verifyStoppedHandler.Handle)
}

// registerTestTools registers test-specific tools
func (r *Registry) registerTestTools() {
	// Create test handlers
	healthCheckHandler := testing.NewHealthCheckHandler(r.processManager)
	processInfoHandler := testing.NewProcessInfoHandler(r.processManager)
	portFinderHandler := testing.NewPortFinderHandler(r.portManager)
	waitReadyHandler := testing.NewWaitReadyHandler(r.processManager)
	
	// Register test tools
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__health_check",
		Description: "Check health status of FTL process (test tool)",
	}, healthCheckHandler.Handle)
	
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__process_info",
		Description: "Get detailed process tree information (test tool)",
	}, processInfoHandler.Handle)
	
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__port_finder",
		Description: "Find available port in range (test tool)",
	}, portFinderHandler.Handle)
	
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__wait_ready",
		Description: "Wait for process to be ready with polling (test tool)",
	}, waitReadyHandler.Handle)
	
	// Register test configuration tools - create handlers first
	getTestConfigHandler := testing.NewGetTestConfigHandler()
	updateTestConfigHandler := testing.NewUpdateTestConfigHandler()
	createTestProjectHandler := testing.NewCreateTestProjectHandler()
	cleanupTestDataHandler := testing.NewCleanupTestDataHandler()
	
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__get_test_config",
		Description: "Get current test configuration (test tool)",
	}, getTestConfigHandler.Handle)
	
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__update_test_config",
		Description: "Update test configuration (test tool)",
	}, updateTestConfigHandler.Handle)
	
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__create_test_project",
		Description: "Create standardized test project (test tool)",
	}, createTestProjectHandler.Handle)
	
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__cleanup_test_data",
		Description: "Clean up test data and reset configuration (test tool)",
	}, cleanupTestDataHandler.Handle)

	// Register TestHelpers integration tools
	serverStartHandler := testing.NewServerStartHandler()
	serverStopHandler := testing.NewServerStopHandler()
	serverStatusHandler := testing.NewServerStatusHandler()
	testProjectSetupHandler := testing.NewTestProjectSetupHandler()
	ensureTestDirectoryHandler := testing.NewEnsureTestDirectoryHandler()
	generateTestProjectNameHandler := testing.NewGenerateTestProjectNameHandler()
	cleanupTestProjectsHandler := testing.NewCleanupTestProjectsHandler()

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__server_start",
		Description: "Start FTL development server for testing (TestHelpers integration)",
	}, serverStartHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__server_stop",
		Description: "Stop running FTL development server (TestHelpers integration)",
	}, serverStopHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__server_status",
		Description: "Check if FTL development server is running (TestHelpers integration)",
	}, serverStatusHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__setup_test_projects",
		Description: "Setup test projects configuration (replaces TestHelpers.resetTestProjectsFile)",
	}, testProjectSetupHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__ensure_test_directory",
		Description: "Ensure test directory exists with FTL project (replaces TestHelpers.ensureTestDirectory)",
	}, ensureTestDirectoryHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__generate_test_project_name",
		Description: "Generate unique test project name (replaces TestHelpers.generateTestProjectName)",
	}, generateTestProjectNameHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "mcp-server__cleanup_test_projects",
		Description: "Comprehensive test project cleanup (replaces TestHelpers.cleanupTestData)",
	}, cleanupTestProjectsHandler.Handle)
}