package html

import (
	"fmt"

	"github.com/fastertools/ftl/internal/models"
	ui "github.com/fastertools/ftl/ui/gen"
)

// GenerateProcessHTML generates HTML for a single process status section
func GenerateProcessHTML(processName string, processInfo models.ProcessInfo, colorScheme string) string {
	var dotColor, statusText, statusColor, statusBgColor string

	if processInfo.IsRunning {
		if colorScheme == "green" {
			dotColor = ui.CSSClasses.StatusDots["running"]
			statusText = ui.StatusRunning
			statusColor = ui.CSSClasses.Text["success"]
			statusBgColor = "bg-green-900/50"
		} else { // blue for watch mode
			dotColor = ui.CSSClasses.StatusDots["building"]
			statusText = ui.StatusRunning
			statusColor = ui.CSSClasses.Text["info"]
			statusBgColor = "bg-blue-900/50"
		}
	} else {
		dotColor = ui.CSSClasses.StatusDots["stopped"]
		statusText = ui.StatusStopped
		if processName == ui.ProcessNameWatch {
			statusText = ui.StatusInactive
		}
		statusColor = ui.CSSClasses.Text["muted"]
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
		(map[bool]string{true: ui.CSSClasses.Text["primary"], false: ui.CSSClasses.Text["muted"]})[processInfo.IsRunning],
		processName, statusBgColor, statusColor, statusText)
}

// GenerateIntegratedProcessHTML creates complete process status with controls
func GenerateIntegratedProcessHTML(status models.DetailedStatusInfo, projectPath string) string {
	if status.ActiveProcess != nil && status.ActiveProcess.IsRunning {
		// Display the active process
		displayName := ui.ProcessNameRegular
		colorScheme := "green"
		if status.ActiveProcess.Type == "watch" {
			displayName = ui.ProcessNameWatch
			colorScheme = "blue"
		} else if status.ActiveProcess.Type == "regular" {
			displayName = ui.ProcessNameRegular
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
	inactiveSection := GenerateProcessSectionHTML("", ui.ProcessNameNone, emptyProcess, projectPath, "gray")

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
			dotColor = ui.CSSClasses.StatusDots["running"]
		} else {
			dotColor = ui.CSSClasses.StatusDots["building"]
		}
	} else {
		dotColor = ui.CSSClasses.StatusDots["stopped"]
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
				style="background-color: %s; border: 1px solid %s; color: %s;"
				onmouseover="this.style.backgroundColor='%s'; this.style.borderColor='%s';"
				onmouseout="this.style.backgroundColor='%s'; this.style.borderColor='%s';"
			>
				Stop ×
			</button>
		`, processType, projectPath,
			ui.Colors.UI["button_bg"], ui.Colors.UI["button_border"], ui.Colors.Accents["red"],
			ui.Colors.UI["button_bg_hover"], ui.Colors.UI["border_hover"],
			ui.Colors.UI["button_bg"], ui.Colors.UI["button_border"])
	}

	return fmt.Sprintf(`
		<div class="flex items-center justify-between p-3 rounded-lg" style="background-color: %s;">
			<div class="flex items-center">
				<div class="w-3 h-3 %s rounded-full mr-3"></div>
				<span class="text-sm font-medium %s">%s</span>
				%s
			</div>
			<div class="flex items-center">
				%s
			</div>
		</div>
	`, ui.Colors.UI["panel_bg"], dotColor,
		(map[bool]string{true: ui.CSSClasses.Text["primary"], false: ui.CSSClasses.Text["muted"]})[processInfo.IsRunning],
		displayName,
		generateProcessDetails(processInfo),
		stopButton)
}

func generateProcessDetails(processInfo models.ProcessInfo) string {
	if processInfo.IsRunning {
		return fmt.Sprintf(`<span class="text-xs ml-3 font-mono" style="color: %s;">PID: <span style="color: %s;">%d</span> | Port: <span style="color: %s;">%d</span></span>`,
			ui.Colors.UI["muted_text"], ui.Colors.Primary.Text, processInfo.PID, ui.Colors.Primary.Text, processInfo.Port)
	}
	return ""
}
