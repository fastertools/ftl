package handlers

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"strconv"
	"strings"

	"github.com/fastertools/ftl/templates"
	"github.com/fastertools/ftl/internal/mcpclient"
)

// calculateToolsHash creates a simple hash of the tools list
func calculateToolsHash(tools []*mcpclient.Tool) string {
	if len(tools) == 0 {
		return "empty"
	}
	// Simple hash: count + concatenated names
	var names []string
	for _, tool := range tools {
		names = append(names, tool.Name)
	}
	return fmt.Sprintf("%d:%s", len(tools), strings.Join(names, ","))
}

// HandleToolsList handles the /tools-list endpoint
func (h *Handler) HandleToolsList(w http.ResponseWriter, r *http.Request) {
	// Get client's current hash
	clientHash := r.FormValue("tools_hash")
	
	// Get the current project from the registry
	currentProject, exists := h.registry.GetCurrentProject()
	if !exists || currentProject == nil {
		// No current project, return empty tools list
		currentHash := "empty"
		if clientHash == currentHash {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		component := templates.ToolsList(templates.Project{}, []*mcpclient.Tool{}, currentHash)
		if err := component.Render(r.Context(), w); err != nil {
			http.Error(w, formatMessage("text-red-500", "Error rendering tools: "+err.Error()), http.StatusInternalServerError)
		}
		return
	}
	
	// Check if there's a running process with a port
	if currentProject.ProcessInfo.ActiveProcess == nil || 
	   !currentProject.ProcessInfo.ActiveProcess.IsRunning || 
	   currentProject.ProcessInfo.ActiveProcess.Port == 0 {
		// No running server, return empty tools list
		currentHash := "empty"
		if clientHash == currentHash {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		project := templates.Project{
			Name: currentProject.Project.Name,
			Path: currentProject.Project.Path,
		}
		component := templates.ToolsList(project, []*mcpclient.Tool{}, currentHash)
		if err := component.Render(r.Context(), w); err != nil {
			http.Error(w, formatMessage("text-red-500", "Error rendering tools: "+err.Error()), http.StatusInternalServerError)
		}
		return
	}
	
	// Query the running server for tools
	tools, err := h.getToolsFromServer(currentProject.ProcessInfo.ActiveProcess.Port)
	if err != nil {
		log.Printf("Error getting tools from server on port %d: %v", currentProject.ProcessInfo.ActiveProcess.Port, err)
		tools = []*mcpclient.Tool{} // Return empty list on error
	}
	
	// Calculate hash of current tools
	currentHash := calculateToolsHash(tools)
	
	// If hash matches, tools haven't changed - don't update DOM
	if clientHash == currentHash {
		w.WriteHeader(http.StatusNoContent) // 204 - HTMX won't update
		return
	}
	
	// Create project for component
	project := templates.Project{
		Name: currentProject.Project.Name,
		Path: currentProject.Project.Path,
	}
	
	// Render the tools component with new hash
	component := templates.ToolsList(project, tools, currentHash)
	if err := component.Render(r.Context(), w); err != nil {
		http.Error(w, formatMessage("text-red-500", "Error rendering tools: "+err.Error()), http.StatusInternalServerError)
		return
	}
}

// HandleToolParams handles the /tool-params/:index endpoint
func (h *Handler) HandleToolParams(w http.ResponseWriter, r *http.Request) {
	// Extract index from URL
	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 3 {
		http.Error(w, "Invalid path", http.StatusBadRequest)
		return
	}
	
	indexStr := pathParts[2]
	index, err := strconv.Atoi(indexStr)
	if err != nil {
		http.Error(w, "Invalid index", http.StatusBadRequest)
		return
	}
	
	// Get the current project from the registry
	currentProject, exists := h.registry.GetCurrentProject()
	if !exists || currentProject == nil {
		http.Error(w, "No current project", http.StatusNotFound)
		return
	}
	
	// Check if there's a running process with a port
	if currentProject.ProcessInfo.ActiveProcess == nil || 
	   !currentProject.ProcessInfo.ActiveProcess.IsRunning || 
	   currentProject.ProcessInfo.ActiveProcess.Port == 0 {
		http.Error(w, "No running server", http.StatusNotFound)
		return
	}
	
	// Query the running server for tools
	tools, err := h.getToolsFromServer(currentProject.ProcessInfo.ActiveProcess.Port)
	if err != nil {
		http.Error(w, "Failed to get tools", http.StatusInternalServerError)
		return
	}
	
	if index < 0 || index >= len(tools) {
		http.Error(w, "Tool not found", http.StatusNotFound)
		return
	}
	
	tool := tools[index]
	
	// Render the tool parameters component
	component := templates.ToolParams(tool, index)
	if err := component.Render(r.Context(), w); err != nil {
		http.Error(w, formatMessage("text-red-500", "Error rendering params: "+err.Error()), http.StatusInternalServerError)
		return
	}
}

// getToolsFromServer queries a specific server for its tools
func (h *Handler) getToolsFromServer(port int) ([]*mcpclient.Tool, error) {
	// Prepare JSON-RPC request
	request := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      1,
		"method":  "tools/list",
	}
	
	requestBody, err := json.Marshal(request)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}
	
	// Make HTTP request to the server
	url := fmt.Sprintf("http://localhost:%d/mcp", port)
	resp, err := http.Post(url, "application/json", bytes.NewReader(requestBody))
	if err != nil {
		return nil, fmt.Errorf("failed to make request: %w", err)
	}
	defer resp.Body.Close()
	
	// Read response
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response: %w", err)
	}
	
	// Parse JSON-RPC response
	var rpcResponse struct {
		Result struct {
			Tools []*mcpclient.Tool `json:"tools"`
		} `json:"result"`
		Error *struct {
			Code    int    `json:"code"`
			Message string `json:"message"`
		} `json:"error"`
	}
	
	if err := json.Unmarshal(body, &rpcResponse); err != nil {
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}
	
	if rpcResponse.Error != nil {
		return nil, fmt.Errorf("RPC error %d: %s", rpcResponse.Error.Code, rpcResponse.Error.Message)
	}
	
	return rpcResponse.Result.Tools, nil
}