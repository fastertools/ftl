package main

import (
	ftl "github.com/fastertools/ftl-cli/sdk/go"
)

// exampleToolHandler processes messages.
// TODO: Replace this with your actual tool implementation.
func exampleToolHandler(input map[string]interface{}) ftl.ToolResponse {
	message, ok := input["message"].(string)
	if !ok {
		return ftl.Error("Invalid input: message must be a string")
	}
	
	// TODO: Implement your tool logic here
	return ftl.Text("Processed: " + message)
}

func main() {
	// Define your tools
	tools := map[string]ftl.ToolDefinition{
		// Replace 'exampleTool' with your actual tool name
		"exampleTool": {
			Description: "An example tool that processes messages",
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
			Handler: exampleToolHandler,
		},
		
		// Add more tools here as needed:
		// "anotherTool": {
		//     Description: "Another tool description",
		//     InputSchema: map[string]interface{}{
		//         "type": "object",
		//         "properties": map[string]interface{}{
		//             "param": map[string]interface{}{"type": "string"},
		//         },
		//     },
		//     Handler: anotherHandler,
		// },
	}
	
	// Create and start the handler
	ftl.CreateTools(tools)
}