package tools

import (
	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/internal/ftl/files"
	"github.com/fastertools/ftl/internal/ftl/ftl"
	"github.com/fastertools/ftl/internal/ftl/port"
	"github.com/fastertools/ftl/internal/ftl/process"
	"github.com/fastertools/ftl/internal/ftl/tools/lifecycle"
	"github.com/fastertools/ftl/internal/ftl/tools/operations"
)

// Registry manages all MCP tool registrations
type Registry struct {
	server         *mcp.Server
	fileManager    *files.Manager
	processManager *process.Manager
	portManager    *port.Manager
	ftlCommander   *ftl.Commander
}

// NewRegistry creates a new tool registry
func NewRegistry(server *mcp.Server) *Registry {
	// Initialize shared dependencies
	fileManager := files.NewManager()
	processManager := process.NewManager(fileManager)
	portManager := port.NewManager(3000)
	ftlCommander := ftl.NewCommander()

	return &Registry{
		server:         server,
		fileManager:    fileManager,
		processManager: processManager,
		portManager:    portManager,
		ftlCommander:   ftlCommander,
	}
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
		Name:        "ftl-server__up",
		Description: "Run ftl up in regular or watch mode",
	}, upHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "ftl-server__stop",
		Description: "Stop any running FTL process (watch or regular mode)",
	}, stopHandler.HandleStop)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "ftl-server__get_status",
		Description: "Get current status of FTL processes",
	}, statusHandler.Handle)

	// Register operations tools
	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "ftl-server__build",
		Description: "Run ftl build command",
	}, buildHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "ftl-server__list_components",
		Description: "List all components in the FTL project",
	}, componentsHandler.Handle)

	mcp.AddTool(r.server, &mcp.Tool{
		Name:        "ftl-server__get_logs",
		Description: "Get logs from running watch process",
	}, logsHandler.Handle)
}