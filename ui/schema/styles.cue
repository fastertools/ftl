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
	}
	
	// Panel and container styles
	panels: {
		main:      "bg-primary rounded-lg shadow-lg"
		sidebar:   "bg-secondary rounded-lg"
		activity:  "bg-primary rounded-lg"
		control:   "bg-secondary rounded-lg p-4"
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