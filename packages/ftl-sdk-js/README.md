# @fastertools/ftl-sdk-js

JavaScript SDK for building FTL MCP (Model Context Protocol) tools.

## Installation

```bash
npm install @fastertools/ftl-sdk-js
```

## Usage

```javascript
import { createHandler } from '@fastertools/ftl-sdk-js';

const handler = createHandler({
  name: 'my-tool',
  version: '1.0.0',
  description: 'My awesome FTL tool',
  tools: {
    myTool: {
      description: 'Does something useful',
      parameters: {
        input: { type: 'string', required: true }
      },
      execute: async ({ input }) => {
        return `Processed: ${input}`;
      }
    }
  }
});

export default handler;
```

## Documentation

For full documentation, visit [https://github.com/anthropics/ftl-cli](https://github.com/anthropics/ftl-cli)

## License

Apache-2.0