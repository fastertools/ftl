package ui

// Color definitions for consistent theming
#Colors: {
	// Base colors
	primary: {
		bg:   "#1a1a1a"
		text: "#ffffff"
	}
	secondary: {
		bg:   "#2a2a2a"
		text: "#a0a0a0"
	}
	
	// UI-specific colors for components
	ui: {
		// Backgrounds
		sidebar_bg:       "#1e1e1e"    // Sidebar specific background
		panel_bg:         "#333333"    // Panel/card backgrounds
		button_bg:        "#4a4a4a"    // Default button background
		button_bg_hover:  "#5a5a5a"    // Button hover background
		tool_param_bg:    "#2a2a2a"    // Tool parameter background
		
		// Borders
		border_primary:   "#3a3a3a"    // Primary border color
		border_hover:     "#6a6a6a"    // Hover border color
		button_border:    "#5a5a5a"    // Button border color
		
		// Text colors
		muted_text:       "#a0a0a0"    // Muted/secondary text
		secondary_text:   "#666666"    // Even more muted text
		tool_label_text:  "#888888"    // Tool parameter labels
		header_bg:        "#2a2a2a"    // Header background color
	}
	
	// Accent colors
	accents: {
		green:  "#22c55e"
		blue:   "#3b82f6"
		orange: "#f97316"
		red:    "#ef4444"
		yellow: "#f59e0b"
		gray:   "#666666"
	}
	
	// Status-specific colors
	status: {
		running:  "#22c55e"
		stopped:  "#666666"
		error:    "#ef4444"
		warning:  "#f97316"
		inactive: "#666666"
		building: "#3b82f6"
		starting: "#f59e0b"
	}
}

// CSS class mappings for Tailwind
#CSSClasses: {
	// Text color classes
	text: {
		success: "text-green-400"
		error:   "text-red-400"
		info:    "text-blue-400"
		warning: "text-orange-400"
		muted:   "text-gray-400"
		primary: "text-white"
	}
	
	// Background color classes for status dots
	status_dots: {
		running:  "bg-green-500"
		stopped:  "bg-gray-500"
		error:    "bg-red-500"
		warning:  "bg-orange-500"
		building: "bg-blue-500"
		inactive: "bg-gray-500"
	}
	
	// Button styles
	buttons: {
		primary:   "bg-blue-600 hover:bg-blue-700 text-white"
		secondary: "bg-gray-600 hover:bg-gray-700 text-white"
		danger:    "bg-red-600 hover:bg-red-700 text-white"
		success:   "bg-green-600 hover:bg-green-700 text-white"
		ghost:     "hover:bg-gray-700 text-gray-300"
		
		// FTL Control buttons - uses custom colors from ui section
		ftl_control: "w-full text-white text-sm py-2.5 px-4 rounded-md transition-all duration-200 font-medium"
		
		// Small action buttons (like Parameters)
		action_small: "text-xs px-2 py-1 rounded hover:bg-gray-600 transition-colors bg-gray-700 text-gray-300"
		
		// Clear/utility buttons
		utility: "text-xs px-2 py-1 rounded hover:bg-gray-800 transition-colors text-gray-300"
	}
	
	// Panel and container styles
	panels: {
		main:      "bg-primary rounded-lg shadow-lg"
		sidebar:   "bg-secondary rounded-lg"
		activity:  "bg-primary rounded-lg"
		control:   "bg-secondary rounded-lg p-4"
	}
	
	// Component classes - frequently repeated patterns
	components: {
		// Typography
		heading_lg:     "text-lg font-semibold mb-4 text-white"
		heading_sm:     "text-sm font-medium mb-2"
		text_label:     "text-sm"
		text_value:     "text-sm text-white font-medium"
		text_mono:      "text-sm text-white font-mono"
		text_mono_xs:   "text-sm text-white font-mono text-xs"
		
		// Layout containers
		space_y_4:      "space-y-4"
		space_y_2:      "space-y-2"
		flex_between:   "flex justify-between"
		flex_center:    "flex items-center"
		flex_space_3:   "flex items-center space-x-3"
		
		// Grid layouts
		grid_3_cols:    "grid grid-cols-3 gap-3"
		grid_4_cols:    "grid grid-cols-4 gap-3"
		
		// Status indicators
		status_dot:     "w-2 h-2 rounded-full"
		
		// Process status containers
		process_item:   "flex items-center justify-between p-2 rounded"
		
		// Section dividers
		border_top:     "border-t pt-4"
		
		// Control panels
		control_panel:  "p-4 rounded-lg"
	}
	
	// Composite components (combinations of classes)
	composite: {
		// Complete heading with muted text
		section_heading: "text-sm font-medium mb-3"
		
		// Process status row
		process_row: "flex items-center justify-between p-2 rounded"
		
		// Info label-value pair
		info_row: "flex justify-between"
	}
}

// Color schemes for different process states
#ProcessColorSchemes: {
	regular: {
		dot_class:   #CSSClasses.status_dots.running
		text_class:  #CSSClasses.text.success
		hex_color:   #Colors.status.running
		label:       "green"
	}
	watch: {
		dot_class:   "bg-blue-500"
		text_class:  #CSSClasses.text.info
		hex_color:   #Colors.accents.blue
		label:       "blue"
	}
	stopped: {
		dot_class:   #CSSClasses.status_dots.stopped
		text_class:  #CSSClasses.text.muted
		hex_color:   #Colors.status.stopped
		label:       "gray"
	}
	error: {
		dot_class:   #CSSClasses.status_dots.error
		text_class:  #CSSClasses.text.error
		hex_color:   #Colors.status.error
		label:       "red"
	}
	building: {
		dot_class:   #CSSClasses.status_dots.building
		text_class:  #CSSClasses.text.info
		hex_color:   #Colors.status.building
		label:       "blue"
	}
}

// Message formatting styles
#MessageStyles: {
	// CLI message prefixes and colors
	cli: {
		info: {
			prefix: "[INFO]"
			color:  "blue"
		}
		error: {
			prefix: "[ERROR]"
			color:  "red"
		}
		warning: {
			prefix: "[WARN]"
			color:  "yellow"
		}
		success: {
			prefix: "[✓]"
			color:  "green"
		}
		debug: {
			prefix: "[DEBUG]"
			color:  "gray"
		}
	}
	
	// Web message styling
	web: {
		info: {
			class:  #CSSClasses.text.info
			prefix: "ℹ"
		}
		error: {
			class:  #CSSClasses.text.error
			prefix: "✗"
		}
		warning: {
			class:  #CSSClasses.text.warning
			prefix: "⚠"
		}
		success: {
			class:  #CSSClasses.text.success
			prefix: "✓"
		}
	}
}