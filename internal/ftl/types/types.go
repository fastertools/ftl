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