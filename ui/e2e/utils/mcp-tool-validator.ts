import { spawn } from 'child_process';
import { writeFileSync, unlinkSync, readFileSync } from 'fs';
import { join } from 'path';

export interface ValidationResult {
    valid: boolean;
    errors: string[];
    warnings?: string[];
}

export interface ValidationOptions {
    timeout?: number; // Timeout in milliseconds
    cueCommand?: string; // Path to CUE binary (defaults to 'cue')
    verbose?: boolean; // Include detailed error context
}

/**
 * CUE Schema Validator for MCP Tool Inputs
 * 
 * Uses CUE CLI as subprocess for reliable validation against the comprehensive
 * schemas defined in /mcp-server/schema/testing.cue
 * 
 * Architecture follows the proven subprocess pattern used in the Go validation
 * system (validation/validation.go) for consistency.
 */
export class MCPToolValidator {
    private schemaDir: string;
    private projectRoot: string;
    private cueCommand: string;
    private defaultTimeout: number;

    constructor(options: ValidationOptions = {}) {
        this.projectRoot = join(__dirname, '..', '..', '..');
        this.schemaDir = join(this.projectRoot, 'mcp-server', 'schema');
        this.cueCommand = options.cueCommand || 'cue';
        this.defaultTimeout = options.timeout || 10000; // 10 second default timeout
    }

    /**
     * Validate MCP tool input against its CUE schema
     */
    async validateToolInput(
        toolName: string, 
        input: unknown, 
        options: ValidationOptions = {}
    ): Promise<ValidationResult> {
        const schemaName = this.getSchemaName(toolName);
        if (!schemaName) {
            return {
                valid: false,
                errors: [`Unknown MCP tool: ${toolName}. No schema definition found.`]
            };
        }

        try {
            return await this.validateWithCUE(input, schemaName, options);
        } catch (error) {
            return {
                valid: false,
                errors: [`Validation failed: ${error instanceof Error ? error.message : String(error)}`]
            };
        }
    }

    /**
     * Validate input against a specific CUE schema definition
     */
    async validateWithSchema(
        input: unknown,
        schemaDefinition: string,
        options: ValidationOptions = {}
    ): Promise<ValidationResult> {
        try {
            return await this.validateWithCUE(input, schemaDefinition, options);
        } catch (error) {
            return {
                valid: false,
                errors: [`Schema validation failed: ${error instanceof Error ? error.message : String(error)}`]
            };
        }
    }

    /**
     * Get available tool schemas for documentation/testing
     */
    getAvailableToolSchemas(): Record<string, string> {
        return {
            // Standard FTL tools
            'mcp-server__init': '#InitToolInput',
            'mcp-server__add': '#AddToolInput',
            'mcp-server__build': '#BuildToolInput',
            'mcp-server__up': '#UpToolInput',
            'mcp-server__stop': '#StopToolInput',
            'mcp-server__status': '#StatusToolInput',
            'mcp-server__logs': '#LogsToolInput',
            
            // Test-specific tools
            'mcp-server__server_start': '#ServerStartInput',
            'mcp-server__server_stop': '#ServerStopInput',
            'mcp-server__server_status': '#ServerStatusInput',
            'mcp-server__generate_test_project_name': '#GenerateTestProjectNameInput',
            'mcp-server__ensure_test_directory': '#EnsureTestDirectoryInput',
            'mcp-server__cleanup_test_projects': '#CleanupTestProjectsInput',
            'mcp-server__wait_ready': '#WaitReadyInput',
            'mcp-server__health_check': '#HealthCheckInput',
            'mcp-server__process_info': '#ProcessInfoInput'
        };
    }

    /**
     * Internal: Get CUE schema name for MCP tool
     */
    private getSchemaName(toolName: string): string | null {
        const schemas = this.getAvailableToolSchemas();
        return schemas[toolName] || null;
    }

    /**
     * Internal: Perform CUE validation using subprocess
     */
    private async validateWithCUE(
        input: unknown,
        schemaDefinition: string,
        options: ValidationOptions = {}
    ): Promise<ValidationResult> {
        const timeout = options.timeout || this.defaultTimeout;
        const verbose = options.verbose || false;

        // Generate unique temporary filenames
        const timestamp = Date.now();
        const tempDir = '/tmp';
        const dataFile = join(tempDir, `ftl-test-data-${timestamp}.json`);
        const schemaFile = join(tempDir, `ftl-test-schema-${timestamp}.cue`);
        const validationFile = join(tempDir, `ftl-test-validation-${timestamp}.cue`);

        try {
            // Write input data as JSON
            writeFileSync(dataFile, JSON.stringify(input, null, 2));

            // Read the main schema file
            const mainSchemaPath = join(this.schemaDir, 'testing.cue');
            const mainSchema = readFileSync(mainSchemaPath, 'utf8');
            writeFileSync(schemaFile, mainSchema);

            // Create validation file that imports schema and validates data  
            const jsonData = JSON.stringify(input, null, 2);
            const validationContent = `package ftl_mcp

// Import the input data directly as JSON
data: ${jsonData}

// Validate against the specified schema
valid_data: data & ${schemaDefinition}
`;
            writeFileSync(validationFile, validationContent);
            
            // Debug logging for failing tests
            if (verbose) {
                console.log('Validation file content:', validationContent);
            }

            // Run CUE validation with -c flag to show complete errors
            const result = await this.runCUECommand([
                'vet',
                '-c',
                schemaFile,
                validationFile
            ], timeout);

            if (result.exitCode === 0) {
                return { 
                    valid: true, 
                    errors: [],
                    warnings: verbose ? result.stderr.split('\n').filter(line => line.trim()) : undefined
                };
            } else {
                return {
                    valid: false,
                    errors: this.parseValidationErrors(result.stderr, verbose),
                    warnings: verbose ? result.stdout.split('\n').filter(line => line.trim()) : undefined
                };
            }

        } finally {
            // Clean up temporary files
            this.cleanupFile(dataFile);
            this.cleanupFile(schemaFile);
            this.cleanupFile(validationFile);
        }
    }

    /**
     * Internal: Run CUE command with timeout
     */
    private runCUECommand(args: string[], timeout: number): Promise<{exitCode: number, stdout: string, stderr: string}> {
        return new Promise((resolve, reject) => {
            const cue = spawn(this.cueCommand, args);
            let stdout = '';
            let stderr = '';
            
            // Set up timeout
            const timeoutHandle = setTimeout(() => {
                cue.kill('SIGKILL');
                reject(new Error(`CUE validation timed out after ${timeout}ms`));
            }, timeout);

            cue.stdout.on('data', (data) => {
                stdout += data.toString();
            });

            cue.stderr.on('data', (data) => {
                stderr += data.toString();
            });

            cue.on('close', (code) => {
                clearTimeout(timeoutHandle);
                resolve({
                    exitCode: code || 0,
                    stdout: stdout.trim(),
                    stderr: stderr.trim()
                });
            });

            cue.on('error', (error) => {
                clearTimeout(timeoutHandle);
                reject(new Error(`CUE command failed: ${error.message}`));
            });
        });
    }

    /**
     * Internal: Parse CUE validation errors into readable format
     */
    private parseValidationErrors(stderr: string, verbose: boolean = false): string[] {
        if (!stderr.trim()) {
            return ['Validation failed with no specific error message'];
        }

        const lines = stderr.split('\n').filter(line => line.trim());
        
        if (verbose) {
            return lines;
        }

        // Extract key error messages, filtering out file paths and noise
        const errorLines = lines.filter(line => {
            // Keep lines that contain useful error information
            return line.includes('field') || 
                   line.includes('not allowed') ||
                   line.includes('conflicting') ||
                   line.includes('invalid') ||
                   line.includes('required') ||
                   line.includes('expected');
        });

        return errorLines.length > 0 ? errorLines : ['Schema validation failed'];
    }

    /**
     * Internal: Safely clean up temporary file
     */
    private cleanupFile(filePath: string): void {
        try {
            unlinkSync(filePath);
        } catch (error) {
            // Ignore cleanup errors - files might not exist or be locked
        }
    }
}

/**
 * Singleton validator instance for convenience
 * 
 * Usage:
 * import { mcpValidator } from './mcp-tool-validator';
 * const result = await mcpValidator.validateToolInput('mcp-server__init', input);
 */
export const mcpValidator = new MCPToolValidator();

/**
 * Convenience function for quick validation
 */
export async function validateMCPToolInput(
    toolName: string,
    input: unknown,
    options?: ValidationOptions
): Promise<ValidationResult> {
    return await mcpValidator.validateToolInput(toolName, input, options);
}