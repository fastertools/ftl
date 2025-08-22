package models

// APIResponse represents the standard API response structure for JSON endpoints
type APIResponse struct {
	Success bool        `json:"success"`
	Message string      `json:"message"`
	Data    interface{} `json:"data,omitempty"`
	Error   string      `json:"error,omitempty"`
}

// LogResponse represents log polling response data
type LogResponse struct {
	ProjectPath   string       `json:"project_path"`
	ProcessInfo   *ProcessInfo `json:"process_info"`
	NewLogs       string       `json:"new_logs"`
	LogPosition   int          `json:"log_position"`
	HasNewContent bool         `json:"has_new_content"`
}

// ProcessInfo contains information about a running process
type ProcessInfo struct {
	PID       int    `json:"pid"`
	Port      int    `json:"port"`
	IsRunning bool   `json:"is_running"`
	Type      string `json:"type"` // "watch" or "regular"
}

// DetailedStatusInfo contains detailed status for any running FTL process
type DetailedStatusInfo struct {
	ProjectPath   string       `json:"project_path"`
	ActiveProcess *ProcessInfo `json:"active_process,omitempty"` // Single active process (watch or regular)
}

// BuildResponse represents the response from build operations
type BuildResponse struct {
	Success     bool   `json:"success"`
	Output      string `json:"output"`
	Error       string `json:"error,omitempty"`
	ProjectPath string `json:"project_path"`
}

// UpResponse represents the response from up operations
type UpResponse struct {
	Success     bool   `json:"success"`
	Message     string `json:"message"`
	ProjectPath string `json:"project_path"`
	ProcessType string `json:"process_type"`
	PID         int    `json:"pid"`
	Port        int    `json:"port"`
	Error       string `json:"error,omitempty"`
}
