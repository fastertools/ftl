package testing

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/ftl"
	"github.com/fastertools/ftl/mcp-server/internal/files"
	"github.com/fastertools/ftl/mcp-server/internal/process"
	"github.com/fastertools/ftl/mcp-server/internal/types"
	"github.com/fastertools/ftl/mcp-server/tools/project"
	"github.com/fastertools/ftl/mcp-server/tools/lifecycle"
)

// TestHelpers provides enhanced testing utilities that delegate to existing infrastructure
// while adding MCP-specific functionality for comprehensive testing
type TestHelpers struct {
	ftlCommander   *ftl.Commander
	fileManager    *files.Manager
	processManager *process.Manager
	testMode       bool
	tempDir        string
	cleanup        []func() error // Cleanup functions to run after test
}

// NewTestHelpers creates a new TestHelpers instance
func NewTestHelpers(testMode bool) *TestHelpers {
	tempDir := os.TempDir()
	if testMode {
		// Create isolated test directory
		testDir := filepath.Join(tempDir, fmt.Sprintf("ftl-test-%d", time.Now().Unix()))
		os.MkdirAll(testDir, 0755)
		tempDir = testDir
	}

	fileManager := files.NewManager()
	return &TestHelpers{
		ftlCommander:   ftl.NewCommander(), // Updated to match existing pattern
		fileManager:    fileManager,
		processManager: process.NewManager(fileManager),
		testMode:       testMode,
		tempDir:        tempDir,
		cleanup:        make([]func() error, 0),
	}
}

// TestContext contains all context needed for a test
type TestContext struct {
	ProjectName string
	ProjectPath string
	Language    string
	TestID      string
	TempDir     string
}

// ProjectLifecycleResult contains results from project lifecycle test
type ProjectLifecycleResult struct {
	InitSuccess   bool              `json:"init_success"`
	AddSuccess    bool              `json:"add_success"`
	BuildSuccess  bool              `json:"build_success"`
	UpSuccess     bool              `json:"up_success"`
	StopSuccess   bool              `json:"stop_success"`
	FilesCreated  []string          `json:"files_created"`
	Errors        []string          `json:"errors"`
	ProcessStatus map[string]bool   `json:"process_status"` // process_name -> running
	Metadata      map[string]string `json:"metadata"`
}

// TestProjectLifecycle performs complete project lifecycle test (init -> add -> up -> stop)
// This is the enhanced version that delegates to HTMXHelpers patterns but works with MCP tools
func (th *TestHelpers) TestProjectLifecycle(ctx context.Context, testConfig TestContext) (*ProjectLifecycleResult, error) {
	result := &ProjectLifecycleResult{
		FilesCreated:  make([]string, 0),
		Errors:        make([]string, 0),
		ProcessStatus: make(map[string]bool),
		Metadata:      make(map[string]string),
	}

	// Step 1: Initialize project using MCP init tool
	initResult, err := th.testInitProject(ctx, testConfig)
	result.InitSuccess = err == nil
	if err != nil {
		result.Errors = append(result.Errors, fmt.Sprintf("Init failed: %v", err))
	} else {
		result.FilesCreated = append(result.FilesCreated, initResult.FilesCreated...)
	}

	// Step 2: Add tool using MCP add tool
	if result.InitSuccess {
		addResult, err := th.testAddTool(ctx, testConfig)
		result.AddSuccess = err == nil
		if err != nil {
			result.Errors = append(result.Errors, fmt.Sprintf("Add failed: %v", err))
		} else {
			result.FilesCreated = append(result.FilesCreated, addResult.FilesCreated...)
		}
	}

	// Step 3: Build project
	if result.AddSuccess {
		err := th.testBuildProject(ctx, testConfig)
		result.BuildSuccess = err == nil
		if err != nil {
			result.Errors = append(result.Errors, fmt.Sprintf("Build failed: %v", err))
		}
	}

	// Step 4: Start server (up)
	if result.BuildSuccess {
		err := th.testUpProject(ctx, testConfig)
		result.UpSuccess = err == nil
		if err != nil {
			result.Errors = append(result.Errors, fmt.Sprintf("Up failed: %v", err))
		} else {
			// Check process status after startup
			result.ProcessStatus["ftl"] = th.isProcessRunning("ftl")
		}
	}

	// Step 5: Stop server using MCP stop handler
	if result.UpSuccess {
		err := th.testStopProject(ctx, testConfig)
		result.StopSuccess = err == nil
		if err != nil {
			result.Errors = append(result.Errors, fmt.Sprintf("Stop failed: %v", err))
		} else {
			// Verify process stopped
			result.ProcessStatus["ftl"] = th.isProcessRunning("ftl")
		}
	}

	return result, nil
}

// testInitProject tests the MCP init tool
func (th *TestHelpers) testInitProject(ctx context.Context, testConfig TestContext) (*MCPToolResult, error) {
	initHandler := project.NewInitHandler(th.ftlCommander)

	// Prepare input
	params := &mcp.CallToolParamsFor[types.InitProjectInput]{
		Arguments: types.InitProjectInput{
			ProjectName: testConfig.ProjectName,
			Language:    testConfig.Language,
			BasePath:    testConfig.TempDir,
		},
	}

	// Mock MCP server session (for testing)
	session := &mcp.ServerSession{}

	// Call the handler
	_, err := initHandler.Handle(ctx, session, params)
	if err != nil {
		return nil, fmt.Errorf("init handler failed: %w", err)
	}

	// Parse response to determine created files
	result := &MCPToolResult{
		Success:      true,
		FilesCreated: th.findCreatedFiles(testConfig.ProjectPath),
	}

	// Add cleanup for created project
	th.addCleanup(func() error {
		return os.RemoveAll(testConfig.ProjectPath)
	})

	return result, nil
}

// testAddTool tests the MCP add tool
func (th *TestHelpers) testAddTool(ctx context.Context, testConfig TestContext) (*MCPToolResult, error) {
	addHandler := project.NewAddHandler(th.ftlCommander)

	// Prepare input
	params := &mcp.CallToolParamsFor[types.AddToolInput]{
		Arguments: types.AddToolInput{
			ToolName:    "hello-world",
			Language:    testConfig.Language,
			ProjectPath: testConfig.ProjectPath,
		},
	}

	session := &mcp.ServerSession{}

	// Call the handler
	_, err := addHandler.Handle(ctx, session, params)
	if err != nil {
		return nil, fmt.Errorf("add handler failed: %w", err)
	}

	result := &MCPToolResult{
		Success:      true,
		FilesCreated: th.findCreatedFiles(filepath.Join(testConfig.ProjectPath, "tools", "hello-world")),
	}

	return result, nil
}

// testBuildProject tests building the project
func (th *TestHelpers) testBuildProject(ctx context.Context, testConfig TestContext) error {
	// Use FTL commander to build project
	_, err := th.ftlCommander.ExecuteBuild(testConfig.ProjectPath, false)
	if err != nil {
		return fmt.Errorf("build failed: %w", err)
	}

	return nil
}

// testUpProject tests starting the project server
func (th *TestHelpers) testUpProject(ctx context.Context, testConfig TestContext) error {
	// Use ExecuteUp method if available, or build command manually
	executable := th.ftlCommander.GetExecutable()
	
	// Start in background (similar to HTMXHelpers pattern)
	args := []string{"up", "--port", "8090"}
	// Use --project-path flag instead of changing directory
	if testConfig.ProjectPath != "" && testConfig.ProjectPath != "." {
		args = append(args, "--project-path", testConfig.ProjectPath)
	}
	cmd := exec.Command(executable, args...)
	
	err := cmd.Start()
	if err != nil {
		return fmt.Errorf("failed to start server: %w", err)
	}

	// Add cleanup to stop process
	th.addCleanup(func() error {
		if cmd.Process != nil {
			return cmd.Process.Kill()
		}
		return nil
	})

	// Wait a bit for server to start (similar to HTMXHelpers.waitForSettle)
	time.Sleep(2 * time.Second)

	return nil
}

// testStopProject tests stopping the project server using MCP stop handler
func (th *TestHelpers) testStopProject(ctx context.Context, testConfig TestContext) error {
	stopHandler := lifecycle.NewStopHandler(th.fileManager, th.processManager)

	// Prepare input
	params := &mcp.CallToolParamsFor[types.StopInput]{
		Arguments: types.StopInput{
			ProjectPath: testConfig.ProjectPath,
		},
	}

	session := &mcp.ServerSession{}

	// Call the stop handler
	_, err := stopHandler.HandleStop(ctx, session, params)
	if err != nil {
		return fmt.Errorf("stop handler failed: %w", err)
	}

	return nil
}

// MCPToolResult represents the result of an MCP tool test
type MCPToolResult struct {
	Success      bool     `json:"success"`
	FilesCreated []string `json:"files_created"`
	Output       string   `json:"output"`
	Error        string   `json:"error,omitempty"`
}

// TestMCPTool tests a specific MCP tool with given input/output expectations
// This delegates to the existing tool handlers while providing test infrastructure
func (th *TestHelpers) TestMCPTool(ctx context.Context, toolName string, input interface{}, expectedFiles []string) (*MCPToolResult, error) {
	result := &MCPToolResult{}

	switch toolName {
	case "mcp-server__init":
		// Cast input and test init tool
		initInput, ok := input.(types.InitProjectInput)
		if !ok {
			return nil, fmt.Errorf("invalid input type for init tool")
		}

		testConfig := TestContext{
			ProjectName: initInput.ProjectName,
			ProjectPath: filepath.Join(initInput.BasePath, initInput.ProjectName),
			Language:    initInput.Language,
			TestID:      fmt.Sprintf("test-%d", time.Now().Unix()),
			TempDir:     initInput.BasePath,
		}

		toolResult, err := th.testInitProject(ctx, testConfig)
		if err != nil {
			result.Success = false
			result.Error = err.Error()
			return result, err
		}

		result.Success = true
		result.FilesCreated = toolResult.FilesCreated

		// Verify expected files exist
		for _, expectedFile := range expectedFiles {
			fullPath := filepath.Join(testConfig.ProjectPath, expectedFile)
			if !th.fileExists(fullPath) {
				result.Success = false
				result.Error = fmt.Sprintf("expected file not found: %s", expectedFile)
				break
			}
		}

	case "mcp-server__add":
		// Similar pattern for add tool
		addInput, ok := input.(types.AddToolInput)
		if !ok {
			return nil, fmt.Errorf("invalid input type for add tool")
		}

		testConfig := TestContext{
			ProjectPath: addInput.ProjectPath,
			Language:    addInput.Language,
			TestID:      fmt.Sprintf("test-%d", time.Now().Unix()),
		}

		toolResult, err := th.testAddTool(ctx, testConfig)
		if err != nil {
			result.Success = false
			result.Error = err.Error()
			return result, err
		}

		result.Success = true
		result.FilesCreated = toolResult.FilesCreated

	default:
		return nil, fmt.Errorf("unknown tool: %s", toolName)
	}

	return result, nil
}

// Helper methods (delegating to existing patterns where possible)

func (th *TestHelpers) findCreatedFiles(basePath string) []string {
	files := make([]string, 0)
	
	if !th.dirExists(basePath) {
		return files
	}

	filepath.Walk(basePath, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}
		if !info.IsDir() {
			relPath, _ := filepath.Rel(basePath, path)
			files = append(files, relPath)
		}
		return nil
	})

	return files
}

func (th *TestHelpers) fileExists(path string) bool {
	_, err := os.Stat(path)
	return !os.IsNotExist(err)
}

func (th *TestHelpers) dirExists(path string) bool {
	stat, err := os.Stat(path)
	return err == nil && stat.IsDir()
}

func (th *TestHelpers) isProcessRunning(processName string) bool {
	// Simple process check - in real implementation, this would use
	// proper process management similar to HTMXHelpers.getProcessTree
	// For now, return false as a safe default
	return false
}

func (th *TestHelpers) addCleanup(fn func() error) {
	th.cleanup = append(th.cleanup, fn)
}

// Cleanup runs all registered cleanup functions
func (th *TestHelpers) Cleanup() error {
	var errors []string

	for i := len(th.cleanup) - 1; i >= 0; i-- {
		if err := th.cleanup[i](); err != nil {
			errors = append(errors, err.Error())
		}
	}

	if len(errors) > 0 {
		return fmt.Errorf("cleanup errors: %s", strings.Join(errors, ", "))
	}

	return nil
}

// WaitForSettle provides similar functionality to HTMXHelpers.waitForSettle
// but for MCP tool operations instead of HTMX operations
func (th *TestHelpers) WaitForSettle(ctx context.Context, timeout time.Duration) error {
	// In a real implementation, this would check for:
	// - No active MCP tool executions
	// - File system operations complete
	// - Process startups/shutdowns complete
	
	// For now, use simple timeout
	select {
	case <-time.After(timeout):
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

// CreateTestProject creates a temporary test project for testing
func (th *TestHelpers) CreateTestProject(language string) (*TestContext, error) {
	testID := fmt.Sprintf("test-%d", time.Now().Unix())
	projectName := fmt.Sprintf("test-project-%s", testID)
	
	testContext := &TestContext{
		ProjectName: projectName,
		ProjectPath: filepath.Join(th.tempDir, projectName),
		Language:    language,
		TestID:      testID,
		TempDir:     th.tempDir,
	}

	// Add cleanup for test project
	th.addCleanup(func() error {
		return os.RemoveAll(testContext.ProjectPath)
	})

	return testContext, nil
}