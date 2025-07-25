import { createTools, ToolResponse } from 'ftl-sdk'
import * as z from 'zod'

// Define the schema using Zod
const EchoSchema = z.object({
  message: z.string().describe('The input message to process')
})

const handle = createTools({
  echoTs: {
    description: 'An MCP tool written in TypeScript',
    inputSchema: z.toJSONSchema(EchoSchema),
    handler: async (input) => {
      const typedInput = input as z.infer<typeof EchoSchema>
      // TODO: Implement your tool logic here
      return ToolResponse.text(`Processed: ${typedInput.message}`)
    }
  }
})

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request))
})