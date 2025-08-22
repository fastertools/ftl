package html

import (
	"fmt"

	"github.com/fastertools/ftl/internal/models"
)

// GenerateProcessHTML generates HTML for a single process status section
func GenerateProcessHTML(processName string, processInfo models.ProcessInfo, colorScheme string) string {
	var dotColor, statusText, statusColor, statusBgColor string

	if processInfo.IsRunning {
		if colorScheme == "green" {
			dotColor = "bg-green-400"
			statusText = "Running"
			statusColor = "text-green-400"
			statusBgColor = "bg-green-900/50"
		} else { // blue for watch mode
			dotColor = "bg-blue-400"
			statusText = "Active"
			statusColor = "text-blue-400"
			statusBgColor = "bg-blue-900/50"
		}
	} else {
		dotColor = "bg-gray-500"
		statusText = "Stopped"
		if processName == "Watch Mode" {
			statusText = "Inactive"
		}
		statusColor = "text-gray-400"
		statusBgColor = "bg-gray-900/50"
	}

	return fmt.Sprintf(`
		<div class="flex items-center">
			<div class="w-3 h-3 %s rounded-full mr-3"></div>
			<span class="text-sm font-medium %s">%s</span>
		</div>
		<div class="text-right">
			<span class="text-xs px-2 py-1 rounded-full %s %s">%s</span>
		</div>
	`, dotColor,
		(map[bool]string{true: "text-white", false: "text-gray-400"})[processInfo.IsRunning],
		processName, statusBgColor, statusColor, statusText)
}

// GenerateIntegratedProcessHTML creates complete process status with controls
func GenerateIntegratedProcessHTML(status models.DetailedStatusInfo, projectPath string) string {
	if status.ActiveProcess != nil && status.ActiveProcess.IsRunning {
		// Display the active process
		displayName := "FTL Process"
		colorScheme := "green"
		if status.ActiveProcess.Type == "watch" {
			displayName = "Watch Mode"
			colorScheme = "blue"
		} else if status.ActiveProcess.Type == "regular" {
			displayName = "Up Process"
			colorScheme = "green"
		}
		
		activeSection := GenerateProcessSectionHTML(status.ActiveProcess.Type, displayName, *status.ActiveProcess, projectPath, colorScheme)
		
		return fmt.Sprintf(`
			<div class="space-y-3">
				%s
			</div>
		`, activeSection)
	}
	
	// No active process - show inactive state
	emptyProcess := models.ProcessInfo{IsRunning: false}
	inactiveSection := GenerateProcessSectionHTML("", "No Active Process", emptyProcess, projectPath, "gray")
	
	return fmt.Sprintf(`
		<div class="space-y-3">
			%s
		</div>
	`, inactiveSection)
}

// GenerateProcessSectionHTML generates HTML for a process section with controls
func GenerateProcessSectionHTML(processType, displayName string, processInfo models.ProcessInfo, projectPath, colorScheme string) string {
	var dotColor string

	if processInfo.IsRunning {
		if colorScheme == "green" {
			dotColor = "bg-green-400"
		} else {
			dotColor = "bg-blue-400"
		}
	} else {
		dotColor = "bg-gray-500"
	}

	// Generate stop button HTML if process is running
	stopButton := ""
	if processInfo.IsRunning {
		stopButton = fmt.Sprintf(`
			<button 
				hx-post="/mcp"
				hx-target="#ftl-output"
				hx-vals='{"stop_%s": "true", "project-path": "%s"}'
				class="ml-3 px-2 py-1 text-xs rounded-md transition-all duration-200"
				style="background-color: #4a4a4a; border: 1px solid #5a5a5a; color: #ef4444;"
				onmouseover="this.style.backgroundColor='#5a5a5a'; this.style.borderColor='#6a6a6a';"
				onmouseout="this.style.backgroundColor='#4a4a4a'; this.style.borderColor='#5a5a5a';"
			>
				Stop ×
			</button>
		`, processType, projectPath)
	}

	return fmt.Sprintf(`
		<div class="flex items-center justify-between p-3 rounded-lg" style="background-color: #333333;">
			<div class="flex items-center">
				<div class="w-3 h-3 %s rounded-full mr-3"></div>
				<span class="text-sm font-medium %s">%s</span>
				%s
			</div>
			<div class="flex items-center">
				%s
			</div>
		</div>
	`, dotColor,
		(map[bool]string{true: "text-white", false: "text-gray-400"})[processInfo.IsRunning],
		displayName,
		generateProcessDetails(processInfo),
		stopButton)
}

func generateProcessDetails(processInfo models.ProcessInfo) string {
	if processInfo.IsRunning {
		return fmt.Sprintf(`<span class="text-xs ml-3 font-mono" style="color: #a0a0a0;">PID: <span style="color: #ffffff;">%d</span> | Port: <span style="color: #ffffff;">%d</span></span>`,
			processInfo.PID, processInfo.Port)
	}
	return ""
}
