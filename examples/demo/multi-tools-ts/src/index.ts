import { createTools, ToolResponse } from 'ftl-sdk'
import * as z from 'zod'

// Define schemas for each tool
const EchoSchema = z.object({
  message: z.string().describe('The message to echo back')
})

type EchoInput = z.infer<typeof EchoSchema>

const ReverseSchema = z.object({
  text: z.string().describe('The text to reverse')
})

const UppercaseSchema = z.object({
  text: z.string().describe('The text to convert to uppercase')
})

const WordCountSchema = z.object({
  text: z.string().describe('The text to count words in')
})

// Create the multi-tool handler
const handle = createTools({
  echo: {
    description: 'Echo back the input message',
    inputSchema: z.toJSONSchema(EchoSchema),
    handler: (input: EchoInput) => {
      return ToolResponse.text(`Echo: ${input.message}`)
    }
  },
  
  reverseText: {
    name: 'reverse',  // Explicit override to keep it as 'reverse'
    description: 'Reverse the input text',
    inputSchema: z.toJSONSchema(ReverseSchema),
    handler: async (input: z.infer<typeof ReverseSchema>) => {
      return ToolResponse.text(input.text.split('').reverse().join(''))
    }
  },
  
  uppercase: {
    description: 'Convert text to uppercase',
    inputSchema: z.toJSONSchema(UppercaseSchema),
    handler: async (input: z.infer<typeof UppercaseSchema>) => {
      return ToolResponse.text(input.text.toUpperCase())
    }
  },
  
  wordCount: {
    description: 'Count the number of words in the input text',
    inputSchema: z.toJSONSchema(WordCountSchema),
    handler: async (input: z.infer<typeof WordCountSchema>) => {
      const wordCount = input.text.trim().split(/\s+/).filter(word => word.length > 0).length
      return ToolResponse.text(`Word count: ${wordCount}`)
    }
  }
})

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request))
})