import { createTools, ToolResponse } from 'ftl-sdk'
import * as z from 'zod'

// Define the schema using Zod
const ExampleToolSchema = z.object({
  message: z.string().describe('The input message to process')
})

const handle = createTools({
  // Replace 'exampleTool' with your actual tool name
  exampleTool: {
    description: 'An example tool that processes messages',
    inputSchema: z.toJSONSchema(ExampleToolSchema),
    handler: async (input) => {
      const typedInput = input as z.infer<typeof ExampleToolSchema>
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