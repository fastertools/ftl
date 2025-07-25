/**
 * Thin SDK providing MCP protocol types for FTL tool development.
 *
 * This package provides only the type definitions needed to implement
 * MCP-compliant tools. It does not include any HTTP server logic,
 * allowing you to use any web framework of your choice.
 */

import type { JSONSchema } from './json-schema'
export type { JSONSchema } from './json-schema'

/**
 * Tool metadata returned by GET requests to tool endpoints
 */
export interface ToolMetadata {
  /** The name of the tool (must be unique within the gateway) */
  name: string

  /** Optional human-readable title for the tool */
  title?: string

  /** Optional description of what the tool does */
  description?: string

  /** JSON Schema describing the expected input parameters */
  inputSchema: JSONSchema

  /** Optional JSON Schema describing the output format */
  outputSchema?: JSONSchema

  /** Optional annotations providing hints about tool behavior */
  annotations?: ToolAnnotations

  /** Optional metadata for tool-specific extensions */
  _meta?: Record<string, unknown>
}

/**
 * Annotations providing hints about tool behavior
 */
export interface ToolAnnotations {
  /** Optional title annotation */
  title?: string

  /** Hint that the tool is read-only (doesn't modify state) */
  readOnlyHint?: boolean

  /** Hint that the tool may perform destructive operations */
  destructiveHint?: boolean

  /** Hint that the tool is idempotent (same input → same output) */
  idempotentHint?: boolean

  /** Hint that the tool accepts open-world inputs */
  openWorldHint?: boolean
}

/**
 * Response format for tool execution (POST requests)
 */
export interface ToolResponse {
  /** Array of content items returned by the tool */
  content: ToolContent[]

  /** Optional structured content matching the outputSchema */
  structuredContent?: unknown

  /** Indicates if this response represents an error */
  isError?: boolean
}

/**
 * Base type for all content items
 */
export interface BaseContent {
  /** Content type discriminator */
  type: string

  /** Optional annotations for this content */
  annotations?: ContentAnnotations
}

/**
 * Text content
 */
export interface TextContent extends BaseContent {
  type: 'text'

  /** The text content */
  text: string
}

/**
 * Image content
 */
export interface ImageContent extends BaseContent {
  type: 'image'

  /** Base64-encoded image data */
  data: string

  /** MIME type of the image (e.g., "image/png") */
  mimeType: string
}

/**
 * Audio content
 */
export interface AudioContent extends BaseContent {
  type: 'audio'

  /** Base64-encoded audio data */
  data: string

  /** MIME type of the audio (e.g., "audio/wav") */
  mimeType: string
}

/**
 * Resource reference
 */
export interface ResourceContent extends BaseContent {
  type: 'resource'

  /** The resource contents */
  resource: ResourceContents
}

/**
 * Content types that can be returned by tools
 */
export type ToolContent = TextContent | ImageContent | AudioContent | ResourceContent

/**
 * Annotations for content items
 */
export interface ContentAnnotations {
  /** Target audience for this content */
  audience?: string[]

  /** Priority of this content (0.0 to 1.0) */
  priority?: number
}

/**
 * Resource contents for resource-type content
 */
export interface ResourceContents {
  /** URI of the resource */
  uri: string

  /** MIME type of the resource */
  mimeType?: string

  /** Text content of the resource */
  text?: string

  /** Base64-encoded binary content of the resource */
  blob?: string
}

/**
 * Convenience functions for creating responses
 */
export const ToolResponse = {
  /**
   * Create a simple text response
   */
  text(text: string): ToolResponse {
    return {
      content: [
        {
          type: 'text',
          text,
        },
      ],
    }
  },

  /**
   * Create an error response
   */
  error(error: string): ToolResponse {
    return {
      content: [
        {
          type: 'text',
          text: error,
        },
      ],
      isError: true,
    }
  },

  /**
   * Create a response with structured content
   */
  withStructured(text: string, structured: unknown): ToolResponse {
    return {
      content: [
        {
          type: 'text',
          text,
        },
      ],
      structuredContent: structured,
    }
  },
}

/**
 * Convenience functions for creating content items
 */
export const ToolContent = {
  /**
   * Create a text content item
   */
  text(text: string, annotations?: ContentAnnotations): TextContent {
    return {
      type: 'text',
      text,
      annotations,
    }
  },

  /**
   * Create an image content item
   */
  image(data: string, mimeType: string, annotations?: ContentAnnotations): ImageContent {
    return {
      type: 'image',
      data,
      mimeType,
      annotations,
    }
  },

  /**
   * Create an audio content item
   */
  audio(data: string, mimeType: string, annotations?: ContentAnnotations): AudioContent {
    return {
      type: 'audio',
      data,
      mimeType,
      annotations,
    }
  },

  /**
   * Create a resource content item
   */
  resource(resource: ResourceContents, annotations?: ContentAnnotations): ResourceContent {
    return {
      type: 'resource',
      resource,
      annotations,
    }
  },
}

// Type guards for content types
export function isTextContent(content: ToolContent): content is TextContent {
  return content.type === 'text'
}

export function isImageContent(content: ToolContent): content is ImageContent {
  return content.type === 'image'
}

export function isAudioContent(content: ToolContent): content is AudioContent {
  return content.type === 'audio'
}

export function isResourceContent(content: ToolContent): content is ResourceContent {
  return content.type === 'resource'
}

/**
 * Handler function type for tool execution
 */
export type ToolHandler<T = unknown> = (input: T) => ToolResponse | Promise<ToolResponse>

/**
 * Options for creating a tool with metadata
 */
export interface CreateToolOptions<T = unknown> {
  /** Tool metadata */
  metadata: ToolMetadata
  /** Handler function for tool execution */
  handler: ToolHandler<T>
}

/**
 * Tool definition for createTools
 */
export interface ToolDefinition<T = unknown> {
  /** Optional explicit tool name (overrides the property key) */
  name?: string

  /** Optional human-readable title for the tool */
  title?: string

  /** Optional description of what the tool does */
  description?: string

  /** JSON Schema describing the expected input parameters */
  inputSchema: JSONSchema

  /** Optional JSON Schema describing the output format */
  outputSchema?: JSONSchema

  /** Optional annotations providing hints about tool behavior */
  annotations?: ToolAnnotations

  /** Optional metadata for tool-specific extensions */
  _meta?: Record<string, unknown>

  /** Handler function for tool execution */
  handler: ToolHandler<T>
}

/**
 * Converts camelCase to snake_case
 */
function camelToSnake(str: string): string {
  return str.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`).replace(/^_/, '')
}

/**
 * Creates a request handler for multiple MCP tools in a single component.
 *
 * This helper provides a clean way to create a multi-tool component that:
 * - Returns all tool metadata on GET / requests
 * - Routes to specific tools based on the path for POST requests
 * - Handles errors gracefully
 *
 * @example
 * ```typescript
 * import { createTools, ToolResponse } from 'ftl-sdk'
 * import { z } from 'zod'
 *
 * const EchoSchema = z.object({
 *   message: z.string().describe('The message to echo')
 * })
 *
 * const ReverseSchema = z.object({
 *   text: z.string().describe('The text to reverse')
 * })
 *
 * const handle = createTools({
 *   echo: {
 *     description: 'Echo back the input',
 *     inputSchema: zodToJsonSchema(EchoSchema),
 *     handler: async (input: z.infer<typeof EchoSchema>) => {
 *       return ToolResponse.text(`Echo: ${input.message}`)
 *     }
 *   },
 *
 *   reverse: {
 *     description: 'Reverse the input text',
 *     inputSchema: zodToJsonSchema(ReverseSchema),
 *     handler: async (input: z.infer<typeof ReverseSchema>) => {
 *       return ToolResponse.text(input.text.split('').reverse().join(''))
 *     }
 *   }
 * })
 *
 * addEventListener('fetch', (event) => {
 *   event.respondWith(handle(event.request))
 * })
 * ```
 */
export function createTools<T extends Record<string, ToolDefinition<unknown>>>(
  tools: T,
): (request: Request) => Promise<Response> {
  return async function handleRequest(request: Request): Promise<Response> {
    const url = new URL(request.url)
    const path = url.pathname
    const { method } = request

    // Handle metadata request
    if (method === 'GET' && path === '/') {
      const metadata = Object.entries(tools).map(([key, tool]) => ({
        name: tool.name ?? camelToSnake(key),
        title: tool.title,
        description: tool.description,
        inputSchema: tool.inputSchema,
        outputSchema: tool.outputSchema,
        annotations: tool.annotations,
        _meta: tool._meta,
      }))
      return new Response(JSON.stringify(metadata), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    }

    // Handle tool execution
    if (method === 'POST') {
      const toolName = path.slice(1) // Remove leading slash

      // Find the tool by checking both explicit names and converted property keys
      const toolEntry = Object.entries(tools).find(([key, tool]) => {
        const effectiveName = tool.name ?? camelToSnake(key)
        return effectiveName === toolName
      })

      if (toolEntry === undefined) {
        const errorResponse = ToolResponse.error(`Tool '${toolName}' not found`)
        return new Response(JSON.stringify(errorResponse), {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
        })
      }

      const [, tool] = toolEntry

      try {
        const input = await request.json()
        const response = await tool.handler(input)
        return new Response(JSON.stringify(response), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Unknown error'
        const errorResponse = ToolResponse.error(`Tool execution failed: ${errorMessage}`)
        return new Response(JSON.stringify(errorResponse), {
          status: 400,
          headers: { 'Content-Type': 'application/json' },
        })
      }
    }

    // Method not allowed
    return new Response('Method not allowed', {
      status: 405,
      headers: { Allow: 'GET, POST' },
    })
  }
}

/**
 * Creates a request handler configured to handle MCP tool requests.
 *
 * @deprecated Use createTools instead for consistency with multi-tool pattern
 *
 * This helper provides a zero-dependency way to create a tool component that:
 * - Returns metadata on GET requests
 * - Executes the handler on POST requests
 * - Handles errors gracefully
 *
 * @example
 * ```typescript
 * import { createTool, ToolResponse } from 'ftl-sdk'
 *
 * interface EchoRequest {
 *   message: string
 * }
 *
 * const handle = createTool({
 *   metadata: {
 *     name: 'echo',
 *     title: 'Echo Tool',
 *     description: 'Echoes back the input message',
 *     inputSchema: {
 *       type: 'object',
 *       properties: {
 *         message: { type: 'string' }
 *       },
 *       required: ['message']
 *     }
 *   },
 *   handler: async (input: EchoRequest) => {
 *     return ToolResponse.text(`Echo: ${input.message}`)
 *   }
 * })
 *
 * addEventListener('fetch', (event) => {
 *   event.respondWith(handle(event.request))
 * })
 * ```
 *
 */
export function createTool<T = unknown>(
  options: CreateToolOptions<T>,
): (request: Request) => Promise<Response> {
  // For backward compatibility, createTool maintains the original single-tool API
  return async function handleRequest(request: Request): Promise<Response> {
    const { method } = request

    // Handle metadata request
    if (method === 'GET') {
      return new Response(JSON.stringify(options.metadata), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    }

    // Handle tool execution
    if (method === 'POST') {
      try {
        const input = await request.json()
        const response = await options.handler(input as T)
        return new Response(JSON.stringify(response), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Unknown error'
        const errorResponse = ToolResponse.error(`Tool execution failed: ${errorMessage}`)
        return new Response(JSON.stringify(errorResponse), {
          status: 400,
          headers: { 'Content-Type': 'application/json' },
        })
      }
    }

    // Method not allowed
    return new Response('Method not allowed', {
      status: 405,
      headers: { Allow: 'GET, POST' },
    })
  }
}
