# @ftl/sdk-js

JavaScript SDK for building FTL MCP (Model Context Protocol) tools that run on WebAssembly.

## Installation

```bash
npm install @ftl/sdk-js
```

## Quick Start

Create a new FTL tool by extending the `Tool` class:

```javascript
import { ftlTool, Tool, ToolResult, ToolError } from '@ftl/sdk-js';

class MyTool extends Tool {
  get name() {
    return 'my-tool';
  }

  get description() {
    return 'A simple example tool';
  }

  get inputSchema() {
    return {
      type: 'object',
      properties: {
        message: {
          type: 'string',
          description: 'Message to process'
        }
      },
      required: ['message']
    };
  }

  async execute(args) {
    const { message } = args;
    
    if (!message) {
      throw ToolError.invalidArguments('Message is required');
    }
    
    // Process the message
    const result = message.toUpperCase();
    
    // Return the result
    return ToolResult.text(result);
  }
}

// Register the tool
ftlTool(new MyTool());
```

## API Reference

### Tool Class

The base class for all FTL tools. You must extend this class and implement the required methods.

#### Properties to implement:

- `name` (string): The tool's name (used in MCP calls)
- `description` (string): Human-readable description
- `inputSchema` (object): JSON Schema for input validation

#### Methods to implement:

- `async execute(args)`: Process the tool request and return a `ToolResult`

#### Optional properties:

- `serverName` (string): Override the server name (defaults to `ftl-{toolName}`)
- `serverVersion` (string): Server version (defaults to '0.1.0')
- `capabilities` (object): Additional MCP capabilities

### ToolResult

Factory methods for creating tool results:

- `ToolResult.text(string)`: Return plain text
- `ToolResult.json(object)`: Return JSON data (formatted as text)
- `ToolResult.multi(array)`: Return multiple content blocks

### ToolError

Factory methods for creating tool errors:

- `ToolError.invalidArguments(message)`: Invalid input arguments
- `ToolError.executionError(message, details)`: Execution failed

## MCP Protocol

The SDK automatically handles the MCP protocol, including:

- `initialize`: Protocol handshake
- `tools/list`: List available tools
- `tools/call`: Execute the tool

Your tool is exposed at the `/mcp` endpoint for MCP clients.

## Example: Weather Tool

```javascript
import { ftlTool, Tool, ToolResult, ToolError } from '@ftl/sdk-js';

class WeatherTool extends Tool {
  get name() {
    return 'get-weather';
  }

  get description() {
    return 'Get current weather for a city';
  }

  get inputSchema() {
    return {
      type: 'object',
      properties: {
        city: {
          type: 'string',
          description: 'City name'
        },
        units: {
          type: 'string',
          enum: ['celsius', 'fahrenheit'],
          default: 'celsius'
        }
      },
      required: ['city']
    };
  }

  async execute(args) {
    const { city, units = 'celsius' } = args;
    
    try {
      // Simulate API call
      const weather = await this.fetchWeather(city, units);
      
      return ToolResult.json({
        city,
        temperature: weather.temp,
        conditions: weather.conditions,
        units
      });
    } catch (error) {
      throw ToolError.executionError(
        `Failed to fetch weather for ${city}`,
        { originalError: error.message }
      );
    }
  }
  
  async fetchWeather(city, units) {
    // Implementation would go here
    return {
      temp: 22,
      conditions: 'Sunny'
    };
  }
}

ftlTool(new WeatherTool());
```

## Building for WebAssembly

FTL tools are compiled to WebAssembly using Spin. Make sure your `package.json` includes:

```json
{
  "scripts": {
    "build": "webpack --mode=production && spin js2wasm dist/bundle.js -o dist/tool.wasm"
  }
}
```

## License

Apache-2.0