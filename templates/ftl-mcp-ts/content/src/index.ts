import { createTools, ToolResponse } from 'ftl-sdk'
import * as z from 'zod'

// Define the schema using Zod
const {{project-name | pascal_case}}Schema = z.object({
  message: z.string().describe('The input message to process')
})

const handle = createTools({
  {{project-name | camelize}}: {
    title: '{{project-name}}',
    description: '{{tool-description}}',
    inputSchema: z.toJSONSchema({{project-name | pascal_case}}Schema),
    handler: async (input) => {
      const typedInput = input as z.infer<typeof {{project-name | pascal_case}}Schema>
      // TODO: Implement your tool logic here
      return ToolResponse.text(`Processed: ${typedInput.message}`)
    }
  }
  
  // Add more tools here as needed:
  // anotherTool: {
  //   description: 'Another tool description',
  //   inputSchema: z.toJSONSchema(AnotherSchema),
  //   handler: async (input) => { ... }
  // }
})

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request))
})