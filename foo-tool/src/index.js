import { ftlTool, Tool, ToolResult, ToolError } from '@ftl/sdk-js';

/**
 * A tool that does foo
 */
class FooTool extends Tool {
  get name() {
    return 'foo-tool';
  }

  get description() {
    return 'A tool that does foo';
  }

  get inputSchema() {
    return {
      type: 'object',
      properties: {
        input: {
          type: 'string',
          description: 'Input text to process'
        }
      },
      required: ['input']
    };
  }

  execute(args) {
    const { input } = args;
    
    if (!input) {
      throw ToolError.invalidArguments('Input is required');
    }
    
    // TODO: Implement your tool logic here
    const result = `Processed: ${input}`;
    
    return ToolResult.text(result);
  }
}

// Register the tool with FTL
ftlTool(new FooTool());