// Export the handler implementation for componentize-js
export const handler = {
    listTools() {
        return [{
            name: 'javascript_test',
            description: 'An MCP tool written in JavaScript',
            inputSchema: JSON.stringify({
                type: 'object',
                properties: {
                    input: { 
                        type: 'string', 
                        description: 'Input to process' 
                    }
                },
                required: ['input']
            })
        }];
    },

    callTool(name, argumentsStr) {
        let args;
        try {
            args = JSON.parse(argumentsStr);
        } catch (e) {
            return {
                tag: 'error',
                val: {
                    code: -32602,
                    message: `Invalid JSON arguments: ${e}`,
                    data: undefined
                }
            };
        }

        switch (name) {
            case 'javascript_test': {
                const input = args.input || 'No input provided';
                // TODO: Implement your tool logic here
                return {
                    tag: 'text',
                    val: `Processed: ${input}`
                };
            }
            default:
                return {
                    tag: 'error',
                    val: {
                        code: -32601,
                        message: `Unknown tool: ${name}`,
                        data: undefined
                    }
                };
        }
    },

    listResources() {
        return [
            // Add resources here
            // Example:
            // {
            //     uri: 'example://resource',
            //     name: 'Example Resource',
            //     description: 'An example resource',
            //     mimeType: 'text/plain'
            // }
        ];
    },

    readResource(uri) {
        // According to WIT, this returns ResourceContents, not a Result
        // So we need to return a valid ResourceContents or throw
        throw {
            code: -32601,
            message: `Resource not found: ${uri}`,
            data: undefined
        };
    },

    listPrompts() {
        return [
            // Add prompts here
            // Example:
            // {
            //     name: 'greeting',
            //     description: 'Generate a greeting',
            //     arguments: [{
            //         name: 'name',
            //         description: 'Name to greet',
            //         required: true
            //     }]
            // }
        ];
    },

    getPrompt(name, argumentsStr) {
        // According to WIT, this returns Array<PromptMessage>, not a Result
        // So we need to return an array or throw
        throw {
            code: -32601,
            message: `Prompt not found: ${name}`,
            data: undefined
        };
    }
};