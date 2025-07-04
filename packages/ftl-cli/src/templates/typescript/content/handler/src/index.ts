// Export the handler implementation for componentize-js
export const handler = {
    listTools() {
        return [{
            name: '{{project-name | snake_case}}',
            description: '{{project-description}}',
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

    callTool(name: string, argumentsStr: string) {
        let args: any;
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
            case '{{project-name | snake_case}}': {
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
        return [];
    },

    readResource(uri: string) {
        // According to WIT, this returns ResourceContents, not a Result
        // So we need to return a valid ResourceContents or throw
        throw {
            code: -32601,
            message: `Resource not found: ${uri}`,
            data: undefined
        };
    },

    listPrompts() {
        return [];
    },

    getPrompt(name: string, argumentsStr: string) {
        // According to WIT, this returns Array<PromptMessage>, not a Result
        // So we need to return an array or throw
        throw {
            code: -32601,
            message: `Prompt not found: ${name}`,
            data: undefined
        };
    }
};