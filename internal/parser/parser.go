package parser

import (
	"encoding/json"
	"log"

	"github.com/fastertools/ftl/internal/models"
)

// ParseDetailedStatusResponse parses the detailed JSON response from MCP server
func ParseDetailedStatusResponse(result, projectPath string) models.DetailedStatusInfo {
	log.Printf("DEBUG: ParseDetailedStatusResponse - Raw input: %q", result)
	log.Printf("DEBUG: ParseDetailedStatusResponse - Input length: %d", len(result))
	
	var detailedStatus models.DetailedStatusInfo
	
	// Try to parse as JSON
	if err := json.Unmarshal([]byte(result), &detailedStatus); err != nil {
		log.Printf("ERROR: Failed to parse detailed status JSON: %v, raw result: %q", err, result)
		// Return default values on parse error
		return models.DetailedStatusInfo{
			ProjectPath:   projectPath,
			ActiveProcess: nil, // No active process on parse error
		}
	}
	
	log.Printf("DEBUG: ParseDetailedStatusResponse - Successfully parsed: %+v", detailedStatus)
	return detailedStatus
}

// ParseLogResponse parses the MCP server JSON response and extracts structured log data
func ParseLogResponse(result, projectPath string, since int) models.LogResponse {
	log.Printf("DEBUG: ParseLogResponse - Raw input: %q", result)
	log.Printf("DEBUG: ParseLogResponse - Input length: %d", len(result))
	
	logResp := models.LogResponse{
		ProjectPath:   projectPath,
		NewLogs:       "",
		LogPosition:   since,
		HasNewContent: false,
	}
	
	// Parse the JSON response from the MCP server
	var mcpLogResponse struct {
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
	
	if err := json.Unmarshal([]byte(result), &mcpLogResponse); err != nil {
		log.Printf("ERROR: Failed to parse log response JSON: %v, raw result: %q", err, result)
		return logResp
	}
	
	log.Printf("DEBUG: ParseLogResponse - Successfully parsed: Success=%t, NewLogsCount=%d, Error=%q", 
		mcpLogResponse.Success, mcpLogResponse.NewLogsCount, mcpLogResponse.Error)
	
	// Check if the response was successful
	if !mcpLogResponse.Success {
		log.Printf("WARNING: Log response was not successful: %s", mcpLogResponse.Error)
		return logResp
	}
	
	// Map the MCP response to our internal models
	logResp.ProcessInfo = &models.ProcessInfo{
		PID:       mcpLogResponse.PID,
		Port:      mcpLogResponse.Port,
		IsRunning: mcpLogResponse.IsRunning,
		Type:      mcpLogResponse.ProcessType,
	}
	
	// Check if we have new logs
	if mcpLogResponse.NewLogsCount > 0 && mcpLogResponse.Logs != "" {
		logResp.NewLogs = mcpLogResponse.Logs
		logResp.LogPosition = mcpLogResponse.TotalLines
		logResp.HasNewContent = true
	}
	
	return logResp
}

// ParseBuildResponse parses build response from MCP server
func ParseBuildResponse(result string) models.BuildResponse {
	log.Printf("DEBUG: ParseBuildResponse - Raw input: %q", result)
	
	var buildResponse models.BuildResponse
	
	if err := json.Unmarshal([]byte(result), &buildResponse); err != nil {
		log.Printf("ERROR: Failed to parse build response JSON: %v, raw result: %q", err, result)
		return models.BuildResponse{
			Success: false,
			Error:   "Failed to parse build response",
			Output:  "",
		}
	}
	
	log.Printf("DEBUG: ParseBuildResponse - Successfully parsed: Success=%t", buildResponse.Success)
	return buildResponse
}

// ParseUpResponse parses up response from MCP server
func ParseUpResponse(result string) models.UpResponse {
	log.Printf("DEBUG: ParseUpResponse - Raw input: %q", result)
	
	var upResponse models.UpResponse
	
	if err := json.Unmarshal([]byte(result), &upResponse); err != nil {
		log.Printf("ERROR: Failed to parse up response JSON: %v, raw result: %q", err, result)
		return models.UpResponse{
			Success: false,
			Error:   "Failed to parse up response",
			Message: "",
		}
	}
	
	log.Printf("DEBUG: ParseUpResponse - Successfully parsed: Success=%t, Message=%q", upResponse.Success, upResponse.Message)
	return upResponse
}

// ParseStatusResponse parses status response from MCP server
func ParseStatusResponse(result, projectPath string) models.ProcessInfo {
	log.Printf("DEBUG: parseStatusResponse called with result: %q", result)
	
	processInfo := models.ProcessInfo{
		PID:       0,
		Port:      0,
		IsRunning: false,
		Type:      "none",
	}
	
	// Parse JSON response from getStatus MCP tool
	var statusData map[string]interface{}
	if err := json.Unmarshal([]byte(result), &statusData); err != nil {
		log.Printf("DEBUG: Failed to parse JSON response: %v", err)
		return processInfo
	}
	
	// Extract values from JSON
	if processType, ok := statusData["process_type"].(string); ok {
		processInfo.Type = processType
		log.Printf("DEBUG: Process type: %s", processType)
	}
	
	if isRunning, ok := statusData["is_running"].(bool); ok {
		processInfo.IsRunning = isRunning
		log.Printf("DEBUG: Is running: %t", isRunning)
	}
	
	if pid, ok := statusData["pid"].(float64); ok {
		processInfo.PID = int(pid)
		log.Printf("DEBUG: PID: %d", processInfo.PID)
	}
	
	if port, ok := statusData["port"].(float64); ok {
		processInfo.Port = int(port)
		log.Printf("DEBUG: Port: %d", processInfo.Port)
	}
	
	log.Printf("DEBUG: Final parsed info: PID=%d, Port=%d, IsRunning=%t, Type=%s", 
		processInfo.PID, processInfo.Port, processInfo.IsRunning, processInfo.Type)
	return processInfo
}

// StopResponse represents the response from stop tools
type StopResponse struct {
	Success     bool   `json:"success"`
	Message     string `json:"message"`
	ProjectPath string `json:"project_path"`
	ProcessType string `json:"process_type"`
	PID         int    `json:"pid"`
	Port        int    `json:"port"`
	Error       string `json:"error,omitempty"`
}

// ParseStopResponse parses the JSON response from stop tools and returns the message
func ParseStopResponse(result string) StopResponse {
	log.Printf("DEBUG: ParseStopResponse - Raw input: %q", result)
	
	var stopResp StopResponse
	
	// Try to parse as JSON
	if err := json.Unmarshal([]byte(result), &stopResp); err != nil {
		log.Printf("ERROR: Failed to parse stop response JSON: %v, raw result: %q", err, result)
		// Return default response on parse error
		return StopResponse{
			Success: false,
			Message: "Failed to parse stop response",
			Error:   err.Error(),
		}
	}
	
	log.Printf("DEBUG: ParseStopResponse - Successfully parsed: %+v", stopResp)
	return stopResp
}