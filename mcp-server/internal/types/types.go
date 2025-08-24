package types

// Input types for FTL operations

// FTLUpInput represents input for ftl up command
type FTLUpInput struct {
	ProjectPath string `json:"project_path" jsonschema:"description:Full path to the FTL project directory"`
	Watch       bool   `json:"watch,omitempty" jsonschema:"description:Run in watch mode,default:false"`
	Build       bool   `json:"build,omitempty" jsonschema:"description:Build before running,default:false"`
	Listen      string `json:"listen,omitempty" jsonschema:"description:Listen address (e.g. localhost:3000). If not specified, an available port will be automatically selected"`
}

// GetLogsInput represents input for getting logs
type GetLogsInput struct {
	ProjectPath string `json:"project_path" jsonschema:"description:Full path to the FTL project directory"`
	Since       int    `json:"since,omitempty" jsonschema:"description:Get logs since this line number (0 for all logs),default:0"`
}

// StopInput represents input for stopping any FTL process (watch or regular)
type StopInput struct {
	ProjectPath string `json:"project_path" jsonschema:"description:Full path to the FTL project directory"`
}

// FTLBuildInput represents input for ftl build command
type FTLBuildInput struct {
	ProjectPath string `json:"project_path" jsonschema:"description:Full path to the FTL project directory"`
	Clean       bool   `json:"clean,omitempty" jsonschema:"description:Clean build (rebuild all),default:false"`
}

// GetStatusInput represents input for getting status
type GetStatusInput struct {
	ProjectPath string `json:"project_path" jsonschema:"description:Full path to the FTL project directory"`
	Detailed    bool   `json:"detailed,omitempty" jsonschema:"description:Return detailed status for both regular and watch processes"`
}

// ListComponentsInput represents input for listing components
type ListComponentsInput struct {
	ProjectPath string `json:"project_path" jsonschema:"description:Full path to the FTL project directory"`
}

// ProcessStatus represents the status of an FTL process
type ProcessStatus struct {
	ProcessType string `json:"process_type"` // "watch", "regular", or "none"
	IsRunning   bool   `json:"is_running"`
	PID         int    `json:"pid"`
	Port        int    `json:"port"`
	ProjectPath string `json:"project_path"`
}

// WatchInfo contains information about a watch process
type WatchInfo struct {
	PID         int
	Port        int
	ProjectPath string
	IsRunning   bool
}

// Response types for FTL operations

// LogsResponse represents the response from getting logs
type LogsResponse struct {
	ProjectPath   string `json:"project_path"`
	ProcessType   string `json:"process_type"`
	IsRunning     bool   `json:"is_running"`
	PID           int    `json:"pid"`
	Port          int    `json:"port"`
	Logs          string `json:"logs"`
	TotalLines    int    `json:"total_lines"`
	Since         int    `json:"since"`
	NewLogsCount  int    `json:"new_logs_count"`
	Success       bool   `json:"success"`
	Error         string `json:"error,omitempty"`
}

// UpResponse represents the response from ftl up command
type UpResponse struct {
	Success     bool   `json:"success"`
	Message     string `json:"message"`
	ProjectPath string `json:"project_path"`
	ProcessType string `json:"process_type"`
	PID         int    `json:"pid"`
	Port        int    `json:"port"`
	Error       string `json:"error,omitempty"`
}

// BuildResponse represents the response from ftl build command
type BuildResponse struct {
	Success     bool   `json:"success"`
	Output      string `json:"output"`
	Error       string `json:"error,omitempty"`
	ProjectPath string `json:"project_path"`
}

// StopResponse represents the response from stop commands
type StopResponse struct {
	Success     bool   `json:"success"`
	Message     string `json:"message"`
	ProjectPath string `json:"project_path"`
	ProcessType string `json:"process_type"`
	PID         int    `json:"pid"`
	Port        int    `json:"port"`
	Error       string `json:"error,omitempty"`
}

// Component represents a single FTL component
type Component struct {
	Name        string `json:"name"`
	Language    string `json:"language"`
	Directory   string `json:"directory"`
	Description string `json:"description,omitempty"`
}

// ListComponentsResponse represents the response from listing components
type ListComponentsResponse struct {
	Success     bool        `json:"success"`
	Components  []Component `json:"components"`
	Count       int         `json:"count"`
	ProjectPath string      `json:"project_path"`
	Error       string      `json:"error,omitempty"`
}

// Test tool input types

// HealthCheckInput represents input for health check test tool
type HealthCheckInput struct {
	ProjectPath string `json:"project_path" jsonschema:"description:Full path to the FTL project directory"`
	Timeout     int    `json:"timeout,omitempty" jsonschema:"description:Timeout in seconds,default:30"`
}

// ProcessInfoInput represents input for process info test tool
type ProcessInfoInput struct {
	ProjectPath     string `json:"project_path" jsonschema:"description:Full path to the FTL project directory"`
	IncludeChildren bool   `json:"include_children,omitempty" jsonschema:"description:Include child processes in the tree,default:false"`
}

// PortFinderInput represents input for port finder test tool
type PortFinderInput struct {
	StartPort int `json:"start_port,omitempty" jsonschema:"description:Starting port to search from,default:3000"`
	EndPort   int `json:"end_port,omitempty" jsonschema:"description:Ending port to search to,default:9999"`
}

// WaitReadyInput represents input for wait ready test tool
type WaitReadyInput struct {
	ProjectPath string `json:"project_path" jsonschema:"description:Full path to the FTL project directory"`
	TimeoutSec  int    `json:"timeout_sec,omitempty" jsonschema:"description:Maximum time to wait in seconds,default:30"`
	IntervalSec int    `json:"interval_sec,omitempty" jsonschema:"description:Polling interval in seconds,default:1"`
	MaxAttempts int    `json:"max_attempts,omitempty" jsonschema:"description:Maximum polling attempts,default:30"`
}

// Process management input types

// KillGracefullyInput represents input for kill_gracefully tool
type KillGracefullyInput struct {
	PID         int    `json:"pid,omitempty" jsonschema:"description:Process ID to kill"`
	ProjectPath string `json:"project_path,omitempty" jsonschema:"description:Project path to find and kill process"`
	Timeout     int    `json:"timeout,omitempty" jsonschema:"description:Timeout in seconds before SIGKILL,default:5"`
}

// CleanupOrphansInput represents input for cleanup_orphans tool
type CleanupOrphansInput struct {
	ParentPID    int    `json:"parent_pid,omitempty" jsonschema:"description:Find orphans of this parent PID"`
	ProcessName  string `json:"process_name,omitempty" jsonschema:"description:Find processes by name pattern"`
	Port         int    `json:"port,omitempty" jsonschema:"description:Find processes listening on this port"`
	KillOrphans  bool   `json:"kill_orphans,omitempty" jsonschema:"description:Kill found orphans,default:false"`
}

// VerifyStoppedInput represents input for verify_stopped tool
type VerifyStoppedInput struct {
	PID          int    `json:"pid" jsonschema:"description:Process ID to verify"`
	CleanupPID   bool   `json:"cleanup_pid,omitempty" jsonschema:"description:Clean up PID file if stopped,default:false"`
	ProjectPath  string `json:"project_path,omitempty" jsonschema:"description:Project path for PID file cleanup"`
}

// Test configuration input types

// GetTestConfigInput represents input for get test config tool
type GetTestConfigInput struct {
	Format string `json:"format,omitempty" jsonschema:"description:Output format - json or summary,default:json,enum:json,enum:summary"`
}

// UpdateTestConfigInput represents input for update test config tool
type UpdateTestConfigInput struct {
	Updates map[string]interface{} `json:"updates" jsonschema:"description:Key-value pairs to update in the configuration"`
	Reset   bool                   `json:"reset,omitempty" jsonschema:"description:Reset configuration to defaults before applying updates,default:false"`
}

// CreateTestProjectInput represents input for create test project tool
type CreateTestProjectInput struct {
	Name      string                 `json:"name" jsonschema:"description:Name of the test project"`
	Language  string                 `json:"language,omitempty" jsonschema:"description:Programming language,enum:rust,enum:python,enum:go"`
	Type      string                 `json:"type,omitempty" jsonschema:"description:Project type,default:tool"`
	Overrides map[string]interface{} `json:"overrides,omitempty" jsonschema:"description:Additional project properties to override"`
	CreateDir bool                   `json:"create_dir,omitempty" jsonschema:"description:Create project directory and basic files,default:false"`
}

// CleanupTestDataInput represents input for cleanup test data tool
type CleanupTestDataInput struct {
	KeepProjectsFile bool `json:"keep_projects_file,omitempty" jsonschema:"description:Keep the projects JSON file,default:false"`
	KeepLogs         bool `json:"keep_logs,omitempty" jsonschema:"description:Keep log files,default:false"`
	Force            bool `json:"force,omitempty" jsonschema:"description:Force cleanup without confirmation,default:false"`
}

// Test tool response types

// HealthCheckResponse represents the response from health check
type HealthCheckResponse struct {
	ProjectPath string       `json:"project_path"`
	Healthy     bool         `json:"healthy"`
	ProcessInfo *ProcessInfo `json:"process_info,omitempty"`
	Error       string       `json:"error,omitempty"`
}

// ProcessInfo contains detailed process information
type ProcessInfo struct {
	PID         int    `json:"pid"`
	Port        int    `json:"port"`
	IsRunning   bool   `json:"is_running"`
	Type        string `json:"type"` // "watch" or "regular"
	DeepestPID  int    `json:"deepest_pid,omitempty"`
}

// ProcessTreeResponse represents the response from process info
type ProcessTreeResponse struct {
	ProjectPath string       `json:"project_path"`
	ProcessInfo *ProcessInfo `json:"process_info,omitempty"`
	Children    []int        `json:"children,omitempty"`
	Error       string       `json:"error,omitempty"`
}

// PortFinderResponse represents the response from port finder
type PortFinderResponse struct {
	Port      int    `json:"port"`
	Available bool   `json:"available"`
	Error     string `json:"error,omitempty"`
}

// WaitReadyResponse represents the response from wait ready
type WaitReadyResponse struct {
	ProjectPath string       `json:"project_path"`
	Ready       bool         `json:"ready"`
	Attempts    int          `json:"attempts"`
	ProcessInfo *ProcessInfo `json:"process_info,omitempty"`
	Error       string       `json:"error,omitempty"`
}