package main

import (
	ftl "github.com/fastertools/ftl/sdk/go"
)

func init() {
	ftl.CreateTools(map[string]ftl.ToolDefinition{
		"echo_go": {
			Description: "An MCP tool written in Go",
			InputSchema: map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"message": map[string]interface{}{
						"type":        "string",
						"description": "The input message to process",
					},
				},
				"required": []string{"message"},
			},
			Handler: func(input map[string]interface{}) ftl.ToolResponse {
				message, _ := input["message"].(string)
				return ftl.Textf("Processed: %s", message)
			},
		},
	})
}

func main() {
	// Required by TinyGo but not used
}