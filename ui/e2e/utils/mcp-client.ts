import { spawn, ChildProcess } from 'child_process';
import * as path from 'path';
import { validateMCPToolInput, ValidationResult } from './mcp-tool-validator';

export interface MCPToolResult {
    success: boolean;
    data?: any;
    error?: string;
    raw_output?: string;
}

export interface MCPClientOptions {
    ftlBinaryPath?: string;
    testMode?: boolean;
    timeout?: number;
    validateInputs?: boolean;
}

/**
 * MCP Client for TypeScript Integration
 * 
 * Provides interface to call MCP tools from TypeScript following the same
 * architecture pattern used by the console (embedded MCP server in FTL binary).
 * 
 * Architecture:
 * TypeScript -> MCP Client -> FTL Binary (MCP mode) -> MCP Tools -> FTL Binary (CLI mode)
 * 
 * This mirrors the Go console pattern: Console calls itself via MCP subcommand.
 */
export class MCPClient {
    private ftlBinaryPath: string;
    private testMode: boolean;
    private timeout: number;
    private validateInputs: boolean;

    constructor(options: MCPClientOptions = {}) {
        // Default to project's FTL binary
        const projectRoot = path.resolve(__dirname, '..', '..', '..');
        this.ftlBinaryPath = options.ftlBinaryPath || path.join(projectRoot, 'bin', 'ftl');
        this.testMode = options.testMode || false;
        this.timeout = options.timeout || 30000; // 30 second default
        this.validateInputs = options.validateInputs !== false; // Default to true
    }

    /**
     * Call an MCP tool with input validation
     */
    async callTool(toolName: string, input: any = {}): Promise<MCPToolResult> {
        try {
            // Optional input validation using CUE schemas
            if (this.validateInputs) {
                const validation = await validateMCPToolInput(toolName, input);
                if (!validation.valid) {
                    return {
                        success: false,
                        error: `Input validation failed: ${validation.errors.join(', ')}`
                    };
                }
            }

            // Call MCP tool via FTL binary's embedded MCP server
            return await this.executeMCPCall(toolName, input);

        } catch (error) {
            return {
                success: false,
                error: `MCP tool call failed: ${error instanceof Error ? error.message : String(error)}`
            };
        }
    }

    /**
     * Call an MCP tool without input validation (for testing/debugging)
     */
    async callToolUnsafe(toolName: string, input: any = {}): Promise<MCPToolResult> {
        return await this.executeMCPCall(toolName, input);
    }

    /**
     * Check if MCP server is available
     */
    async checkHealth(): Promise<boolean> {
        try {
            const result = await this.executeMCPCall('mcp-server__health_check', {});
            return result.success;
        } catch (error) {
            return false;
        }
    }

    /**
     * Get available MCP tools
     */
    async listTools(): Promise<string[]> {
        try {
            // This would use a hypothetical list_tools MCP endpoint
            // For now, return the known tools from our validator
            const { mcpValidator } = await import('./mcp-tool-validator');
            return Object.keys(mcpValidator.getAvailableToolSchemas());
        } catch (error) {
            return [];
        }
    }

    /**
     * Internal: Execute MCP tool call via FTL binary
     * 
     * Follows the same pattern as console:
     * - Spawns FTL binary with 'dev mcp' subcommand  
     * - Passes tool call as JSON via stdin
     * - Parses JSON response from stdout
     */
    private async executeMCPCall(toolName: string, input: any): Promise<MCPToolResult> {
        return new Promise((resolve, reject) => {
            // Prepare MCP command args
            const args = ['dev', 'mcp'];
            if (this.testMode) {
                args.push('--test-mode');
            }
            
            // Set up environment
            const env = { ...process.env };
            if (this.testMode) {
                env['FTL_TEST_MODE'] = 'true';
            }

            // Spawn FTL binary in MCP mode
            const mcpProcess = spawn(this.ftlBinaryPath, args, {
                env,
                stdio: ['pipe', 'pipe', 'pipe']
            });

            let stdout = '';
            let stderr = '';
            let resolved = false;

            // Set up timeout
            const timeoutHandle = setTimeout(() => {
                if (!resolved) {
                    resolved = true;
                    mcpProcess.kill('SIGKILL');
                    reject(new Error(`MCP tool call timed out after ${this.timeout}ms`));
                }
            }, this.timeout);

            // Collect output
            mcpProcess.stdout.on('data', (data) => {
                stdout += data.toString();
            });

            mcpProcess.stderr.on('data', (data) => {
                stderr += data.toString();
            });

            mcpProcess.on('close', (code) => {
                clearTimeout(timeoutHandle);
                if (resolved) return;
                resolved = true;

                if (code === 0) {
                    try {
                        // Parse MCP response
                        const response = this.parseMCPResponse(stdout, stderr);
                        resolve(response);
                    } catch (parseError) {
                        resolve({
                            success: false,
                            error: `Failed to parse MCP response: ${parseError}`,
                            raw_output: stdout
                        });
                    }
                } else {
                    resolve({
                        success: false,
                        error: `MCP process exited with code ${code}: ${stderr}`,
                        raw_output: stdout
                    });
                }
            });

            mcpProcess.on('error', (error) => {
                clearTimeout(timeoutHandle);
                if (resolved) return;
                resolved = true;
                reject(new Error(`MCP process error: ${error.message}`));
            });

            // Send MCP tool call request via stdin
            const mcpRequest = {
                method: 'tools/call',
                params: {
                    name: toolName,
                    arguments: input
                }
            };

            mcpProcess.stdin.write(JSON.stringify(mcpRequest) + '\n');
            mcpProcess.stdin.end();
        });
    }

    /**
     * Internal: Parse MCP response from stdout/stderr
     */
    private parseMCPResponse(stdout: string, stderr: string): MCPToolResult {
        // Try to parse as JSON response
        const lines = stdout.trim().split('\n');
        
        // Look for JSON response (typically the last line)
        for (let i = lines.length - 1; i >= 0; i--) {
            const line = lines[i].trim();
            if (line.startsWith('{')) {
                try {
                    const response = JSON.parse(line);
                    
                    // Handle MCP protocol response format
                    if (response.result) {
                        return {
                            success: true,
                            data: response.result,
                            raw_output: stdout
                        };
                    } else if (response.error) {
                        return {
                            success: false,
                            error: response.error.message || String(response.error),
                            raw_output: stdout
                        };
                    }
                } catch (parseError) {
                    // Continue searching for valid JSON
                }
            }
        }

        // If no valid JSON found, return raw output
        return {
            success: stdout.trim().length > 0,
            data: stdout.trim() || null,
            error: stderr.trim() || undefined,
            raw_output: stdout
        };
    }
}

/**
 * Singleton client instance for convenience
 */
export const mcpClient = new MCPClient();

/**
 * Convenience functions for common operations
 */

export async function callMCPTool(toolName: string, input: any = {}): Promise<MCPToolResult> {
    return await mcpClient.callTool(toolName, input);
}

export async function initProject(
    projectName: string, 
    language: string = 'rust', 
    basePath: string = './e2e-projects'
): Promise<MCPToolResult> {
    return await mcpClient.callTool('mcp-server__init', {
        project_name: projectName,
        language,
        base_path: basePath
    });
}

export async function addTool(
    toolName: string,
    language: string,
    projectPath: string
): Promise<MCPToolResult> {
    return await mcpClient.callTool('mcp-server__add', {
        tool_name: toolName,
        language,
        project_path: projectPath
    });
}

export async function startServer(
    projectPath: string,
    port: number = 8080
): Promise<MCPToolResult> {
    return await mcpClient.callTool('mcp-server__up', {
        project_path: projectPath,
        port
    });
}

export async function stopServer(projectPath: string): Promise<MCPToolResult> {
    return await mcpClient.callTool('mcp-server__stop', {
        project_path: projectPath
    });
}

export async function getServerStatus(projectPath: string): Promise<MCPToolResult> {
    return await mcpClient.callTool('mcp-server__status', {
        project_path: projectPath
    });
}

export async function buildProject(projectPath: string): Promise<MCPToolResult> {
    return await mcpClient.callTool('mcp-server__build', {
        project_path: projectPath
    });
}