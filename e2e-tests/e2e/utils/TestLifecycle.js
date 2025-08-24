// TestLifecycle.js - Utilities for test lifecycle management using MCP tools
// Replaces custom JavaScript implementations with calls to Go MCP tools

const { exec } = require('child_process');
const { promisify } = require('util');
const execAsync = promisify(exec);

class TestLifecycle {
  /**
   * Helper method to call MCP tools consistently
   * @private
   */
  static async callMCPTool(toolName, args = {}, mcpServerPath = './mcp-server') {
    try {
      const command = `echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"${toolName}","arguments":${JSON.stringify(args)}},"id":1}' | ${mcpServerPath} --test-mode`;
      
      const { stdout } = await execAsync(command);
      const response = JSON.parse(stdout);
      
      if (response.result && response.result.content) {
        const content = response.result.content[0].text;
        try {
          return JSON.parse(content);
        } catch {
          return content;
        }
      }
      
      return null;
    } catch (error) {
      console.error(`Error calling MCP tool ${toolName}:`, error);
      throw error;
    }
  }

  /**
   * Find an available port using the MCP port finder tool
   * Replaces ServerManager.findAvailablePort() with Go implementation
   * @param {number} startPort - Starting port to search from (default: 8080)
   * @param {string} mcpServerPath - Path to MCP server executable
   * @returns {Promise<number>} - Available port number
   */
  static async findAvailablePort(startPort = 8080, mcpServerPath = './mcp-server') {
    try {
      // Call the MCP port finder tool via command line
      const command = `echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"mcp-server__find_port","arguments":{"start_port":${startPort}}},"id":1}' | ${mcpServerPath} --test-mode`;
      
      const { stdout } = await execAsync(command);
      const response = JSON.parse(stdout);
      
      if (response.result && response.result.content) {
        // Parse the port from the response
        const content = response.result.content[0].text;
        const portMatch = content.match(/port[:\s]+(\d+)/i);
        if (portMatch) {
          return parseInt(portMatch[1]);
        }
      }
      
      // Fallback to default if tool fails
      console.warn('MCP port finder failed, using default port');
      return startPort;
    } catch (error) {
      console.error('Error finding available port:', error);
      return startPort;
    }
  }
  
  /**
   * Wait for server to be ready using MCP wait_ready tool
   * Replaces ServerManager.attemptServerStart() polling logic
   * @param {string} projectPath - Path to the project
   * @param {Object} options - Configuration options
   * @param {number} options.timeout - Maximum wait time in ms (default: 30000)
   * @param {number} options.maxAttempts - Maximum retry attempts (default: 10)
   * @param {string} options.mcpServerPath - Path to MCP server
   * @returns {Promise<boolean>} - True if server is ready
   */
  static async waitForReady(projectPath, options = {}) {
    const timeout = options.timeout || 30000;
    const maxAttempts = options.maxAttempts || 10;
    const mcpServerPath = options.mcpServerPath || './mcp-server';
    
    try {
      // Call the MCP wait_ready tool
      const command = `echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"mcp-server__wait_ready","arguments":{"project_path":"${projectPath}","timeout":${timeout},"max_attempts":${maxAttempts}}},"id":1}' | ${mcpServerPath} --test-mode`;
      
      const { stdout } = await execAsync(command);
      const response = JSON.parse(stdout);
      
      if (response.result && response.result.content) {
        const content = response.result.content[0].text;
        return content.includes('ready') || content.includes('success');
      }
      
      return false;
    } catch (error) {
      console.error('Error waiting for server ready:', error);
      return false;
    }
  }
  
  /**
   * Get process information using MCP process_info tool
   * Replaces ServerManager.cleanupPort() with proper process info
   * @param {string} projectPath - Path to the project
   * @param {string} mcpServerPath - Path to MCP server
   * @returns {Promise<Object>} - Process information object
   */
  static async getProcessInfo(projectPath, mcpServerPath = './mcp-server') {
    try {
      // Call the MCP process_info tool
      const command = `echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"mcp-server__process_info","arguments":{"project_path":"${projectPath}"}},"id":1}' | ${mcpServerPath} --test-mode`;
      
      const { stdout } = await execAsync(command);
      const response = JSON.parse(stdout);
      
      if (response.result && response.result.content) {
        const content = response.result.content[0].text;
        // Parse JSON from the content
        try {
          return JSON.parse(content);
        } catch {
          // If not JSON, return a default structure
          return {
            ftl: { is_running: false },
            watch: { is_running: false },
            active_process: null
          };
        }
      }
      
      return {
        ftl: { is_running: false },
        watch: { is_running: false },
        active_process: null
      };
    } catch (error) {
      console.error('Error getting process info:', error);
      return {
        ftl: { is_running: false },
        watch: { is_running: false },
        active_process: null
      };
    }
  }
  
  /**
   * Check server health using MCP health_check tool
   * @param {string} projectPath - Path to the project
   * @param {Object} options - Configuration options
   * @param {number} options.timeout - Health check timeout in ms
   * @param {string} options.mcpServerPath - Path to MCP server
   * @returns {Promise<Object>} - Health status object
   */
  static async checkHealth(projectPath, options = {}) {
    const timeout = options.timeout || 5000;
    const mcpServerPath = options.mcpServerPath || './mcp-server';
    
    try {
      // Call the MCP health_check tool
      const command = `echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"mcp-server__health_check","arguments":{"project_path":"${projectPath}","timeout":${timeout}}},"id":1}' | ${mcpServerPath} --test-mode`;
      
      const { stdout } = await execAsync(command);
      const response = JSON.parse(stdout);
      
      if (response.result && response.result.content) {
        const content = response.result.content[0].text;
        // Parse health status from content
        try {
          return JSON.parse(content);
        } catch {
          // If not JSON, check for keywords
          return {
            healthy: content.includes('healthy') || content.includes('running'),
            status: content
          };
        }
      }
      
      return {
        healthy: false,
        status: 'unknown'
      };
    } catch (error) {
      console.error('Error checking health:', error);
      return {
        healthy: false,
        status: 'error',
        error: error.message
      };
    }
  }
  
  /**
   * Clean up processes for a project using MCP tools
   * Replaces platform-specific kill commands with proper cleanup
   * @param {string} projectPath - Path to the project
   * @param {number} port - Port to clean up (optional)
   * @returns {Promise<boolean>} - True if cleanup successful
   */
  static async cleanupProcesses(projectPath, port = null) {
    try {
      // First get process info to know what to clean up
      const processInfo = await this.getProcessInfo(projectPath);
      
      // If we have running processes, attempt to stop them gracefully
      if (processInfo.active_process && processInfo.active_process.pid) {
        // Use MCP kill_gracefully tool for cross-platform support
        const killResult = await this.killProcessGracefully(processInfo.active_process.pid, {
          timeout: 3,
          projectPath
        });
        
        if (!killResult.success) {
          console.warn(`Could not kill process ${processInfo.active_process.pid}: ${killResult.message}`);
        }
        
        // Verify the process is actually stopped
        const verifyResult = await this.verifyProcessStopped({
          pid: processInfo.active_process.pid,
          timeout: 5,
          cleanupPid: true,
          projectPath
        });
        
        if (!verifyResult.stopped) {
          console.warn(`Process ${processInfo.active_process.pid} may still be running`);
        }
      }
      
      // Clean up any orphaned processes on the port if specified
      if (port) {
        const cleanupResult = await this.cleanupOrphans({
          port,
          killOrphans: true
        });
        
        if (cleanupResult.killedCount > 0) {
          console.log(`Cleaned up ${cleanupResult.killedCount} orphaned processes on port ${port}`);
        }
      }
      
      return true;
    } catch (error) {
      console.error('Error cleaning up processes:', error);
      return false;
    }
  }
  
  /**
   * Start server with proper lifecycle management
   * Combines port finding, server start, and ready wait
   * @param {string} projectPath - Path to the project
   * @param {Object} options - Configuration options
   * @returns {Promise<Object>} - Server info with port and status
   */
  static async startServer(projectPath, options = {}) {
    try {
      // Find available port
      const port = await this.findAvailablePort(
        options.startPort || 8080,
        options.mcpServerPath
      );
      
      // Start the server (this would typically use spawn or exec)
      // For now, we'll assume the server is started externally
      console.log(`Starting server on port ${port} for project ${projectPath}`);
      
      // Wait for server to be ready
      const isReady = await this.waitForReady(projectPath, {
        timeout: options.timeout || 30000,
        maxAttempts: options.maxAttempts || 10,
        mcpServerPath: options.mcpServerPath
      });
      
      if (!isReady) {
        throw new Error('Server failed to start within timeout');
      }
      
      // Get final process info
      const processInfo = await this.getProcessInfo(projectPath, options.mcpServerPath);
      
      return {
        port,
        ready: true,
        processInfo
      };
    } catch (error) {
      console.error('Error starting server:', error);
      throw error;
    }
  }
  
  /**
   * Stop server with proper cleanup
   * @param {string} projectPath - Path to the project
   * @param {number} port - Server port (optional)
   * @returns {Promise<boolean>} - True if stopped successfully
   */
  static async stopServer(projectPath, port = null) {
    try {
      // Clean up processes
      const cleanupSuccess = await this.cleanupProcesses(projectPath, port);
      
      if (!cleanupSuccess) {
        console.warn('Cleanup may not have completed fully');
      }
      
      // Verify server is stopped
      const processInfo = await this.getProcessInfo(projectPath);
      
      const isStopped = !processInfo.active_process || 
                        !processInfo.active_process.is_running;
      
      if (!isStopped) {
        console.warn('Server may still be running after stop attempt');
      }
      
      return isStopped;
    } catch (error) {
      console.error('Error stopping server:', error);
      return false;
    }
  }

  /**
   * Kill a process gracefully using MCP kill_gracefully tool
   * Tries SIGTERM first, then escalates to SIGKILL if needed
   * @param {number} pid - Process ID to kill
   * @param {Object} options - Kill options
   * @param {number} options.timeout - Timeout in seconds (default: 5)
   * @param {string} options.projectPath - Optional project path to find PID from
   * @param {string} options.mcpServerPath - Path to MCP server
   * @returns {Promise<Object>} - Kill result with success status and method used
   */
  static async killProcessGracefully(pid, options = {}) {
    const timeout = options.timeout || 5;
    const projectPath = options.projectPath || '';
    const mcpServerPath = options.mcpServerPath || './mcp-server';
    
    try {
      const args = pid ? 
        { pid, timeout } : 
        { project_path: projectPath, timeout };
      
      const command = `echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"mcp-server__kill_gracefully","arguments":${JSON.stringify(args)}},"id":1}' | ${mcpServerPath} --test-mode`;
      
      const { stdout } = await execAsync(command);
      const response = JSON.parse(stdout);
      
      if (response.result && response.result.content) {
        const content = response.result.content[0].text;
        try {
          return JSON.parse(content);
        } catch {
          return {
            success: content.includes('success') || content.includes('killed'),
            method: content.includes('SIGKILL') ? 'SIGKILL' : 'SIGTERM',
            message: content
          };
        }
      }
      
      return { success: false, message: 'Kill operation failed' };
    } catch (error) {
      console.error('Error killing process:', error);
      return { success: false, message: error.message };
    }
  }

  /**
   * Clean up orphaned processes using MCP cleanup_orphans tool
   * @param {Object} options - Cleanup options
   * @param {number} options.parentPid - Parent PID to find orphans
   * @param {string} options.processName - Process name to search for
   * @param {number} options.port - Port to find processes on
   * @param {boolean} options.killOrphans - Whether to kill found orphans
   * @param {string} options.mcpServerPath - Path to MCP server
   * @returns {Promise<Object>} - Cleanup result with orphan PIDs and kill count
   */
  static async cleanupOrphans(options = {}) {
    const mcpServerPath = options.mcpServerPath || './mcp-server';
    
    try {
      const args = {};
      if (options.parentPid) args.parent_pid = options.parentPid;
      if (options.processName) args.process_name = options.processName;
      if (options.port) args.port = options.port;
      args.kill_orphans = options.killOrphans || false;
      
      const command = `echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"mcp-server__cleanup_orphans","arguments":${JSON.stringify(args)}},"id":1}' | ${mcpServerPath} --test-mode`;
      
      const { stdout } = await execAsync(command);
      const response = JSON.parse(stdout);
      
      if (response.result && response.result.content) {
        const content = response.result.content[0].text;
        try {
          return JSON.parse(content);
        } catch {
          // Parse from text content
          const orphanMatch = content.match(/orphans?[:\s]+(\[[\d,\s]+\]|\d+)/i);
          const killedMatch = content.match(/killed[:\s]+(\d+)/i);
          return {
            orphanPIDs: orphanMatch ? JSON.parse(orphanMatch[1]) : [],
            killedCount: killedMatch ? parseInt(killedMatch[1]) : 0,
            message: content
          };
        }
      }
      
      return { orphanPIDs: [], killedCount: 0, message: 'No orphans found' };
    } catch (error) {
      console.error('Error cleaning up orphans:', error);
      return { orphanPIDs: [], killedCount: 0, message: error.message };
    }
  }

  /**
   * Verify that a process has stopped using MCP verify_stopped tool
   * @param {Object} options - Verification options
   * @param {number} options.pid - Process ID to verify
   * @param {string} options.projectPath - Project path to find PID from
   * @param {number} options.timeout - Timeout in seconds (default: 10)
   * @param {boolean} options.cleanupPid - Whether to clean up PID file
   * @param {string} options.mcpServerPath - Path to MCP server
   * @returns {Promise<Object>} - Verification result
   */
  static async verifyProcessStopped(options = {}) {
    const timeout = options.timeout || 10;
    const mcpServerPath = options.mcpServerPath || './mcp-server';
    
    try {
      const args = {};
      if (options.pid) args.pid = options.pid;
      if (options.projectPath) args.project_path = options.projectPath;
      args.timeout = timeout;
      args.cleanup_pid = options.cleanupPid || false;
      
      const command = `echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"mcp-server__verify_stopped","arguments":${JSON.stringify(args)}},"id":1}' | ${mcpServerPath} --test-mode`;
      
      const { stdout } = await execAsync(command);
      const response = JSON.parse(stdout);
      
      if (response.result && response.result.content) {
        const content = response.result.content[0].text;
        try {
          return JSON.parse(content);
        } catch {
          return {
            stopped: content.includes('stopped') || content.includes('not running'),
            message: content
          };
        }
      }
      
      return { stopped: false, message: 'Verification failed' };
    } catch (error) {
      console.error('Error verifying process stopped:', error);
      return { stopped: false, message: error.message };
    }
  }

  // ===== TEST CONFIGURATION METHODS =====
  // Phase 6: Test Data Standardization - MCP tool integration

  /**
   * Get test configuration using MCP get_test_config tool
   * Provides standardized test settings across all tests
   * @param {Object} options - Configuration options
   * @param {string} options.format - Format: "json" (full config) or "summary" (key fields only)
   * @param {string} options.mcpServerPath - Path to MCP server
   * @returns {Promise<Object>} - Test configuration object
   */
  static async getTestConfig(options = {}) {
    const format = options.format || 'json';
    const mcpServerPath = options.mcpServerPath || './mcp-server';
    
    try {
      const args = { format };
      const command = `echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"mcp-server__get_test_config","arguments":${JSON.stringify(args)}},"id":1}' | ${mcpServerPath} --test-mode`;
      
      const { stdout } = await execAsync(command);
      const response = JSON.parse(stdout);
      
      if (response.result && response.result.content) {
        const content = response.result.content[0].text;
        try {
          return JSON.parse(content);
        } catch {
          console.error('Failed to parse test config response:', content);
          return null;
        }
      }
      
      return null;
    } catch (error) {
      console.error('Error getting test config:', error);
      throw error;
    }
  }

  /**
   * Update test configuration using MCP update_test_config tool
   * Updates global test settings that affect all test runs
   * @param {Object} updates - Key-value pairs to update in configuration
   * @param {Object} options - Update options
   * @param {boolean} options.reset - Reset to defaults before updating
   * @param {string} options.mcpServerPath - Path to MCP server
   * @returns {Promise<Object>} - Updated configuration object
   */
  static async updateTestConfig(updates, options = {}) {
    const reset = options.reset || false;
    const mcpServerPath = options.mcpServerPath || './mcp-server';
    
    try {
      const args = { updates, reset };
      const command = `echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"mcp-server__update_test_config","arguments":${JSON.stringify(args)}},"id":1}' | ${mcpServerPath} --test-mode`;
      
      const { stdout } = await execAsync(command);
      const response = JSON.parse(stdout);
      
      if (response.result && response.result.content) {
        const content = response.result.content[0].text;
        try {
          const result = JSON.parse(content);
          if (result.success) {
            return result.config;
          } else {
            throw new Error(result.message || 'Update failed');
          }
        } catch (parseError) {
          console.error('Failed to parse update config response:', content);
          throw parseError;
        }
      }
      
      throw new Error('No response from update_test_config tool');
    } catch (error) {
      console.error('Error updating test config:', error);
      throw error;
    }
  }

  /**
   * Create a standardized test project using MCP create_test_project tool
   * Provides consistent project structure across all tests
   * @param {string} name - Project name
   * @param {Object} options - Project creation options
   * @param {string} options.language - Project language (rust, python, go)
   * @param {string} options.type - Project type (default: "tool")
   * @param {boolean} options.createDir - Create project directory and basic files
   * @param {Object} options.overrides - Additional project properties to override
   * @param {string} options.mcpServerPath - Path to MCP server
   * @returns {Promise<Object>} - Created project configuration
   */
  static async createTestProject(name, options = {}) {
    const mcpServerPath = options.mcpServerPath || './mcp-server';
    
    try {
      const args = {
        name,
        language: options.language,
        type: options.type,
        create_dir: options.createDir || false,
        overrides: options.overrides || {}
      };
      
      const command = `echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"mcp-server__create_test_project","arguments":${JSON.stringify(args)}},"id":1}' | ${mcpServerPath} --test-mode`;
      
      const { stdout } = await execAsync(command);
      const response = JSON.parse(stdout);
      
      if (response.result && response.result.content) {
        const content = response.result.content[0].text;
        try {
          const result = JSON.parse(content);
          if (result.success) {
            return result.project;
          } else {
            throw new Error(result.message || 'Project creation failed');
          }
        } catch (parseError) {
          console.error('Failed to parse create project response:', content);
          throw parseError;
        }
      }
      
      throw new Error('No response from create_test_project tool');
    } catch (error) {
      console.error('Error creating test project:', error);
      throw error;
    }
  }

  /**
   * Clean up all test data using MCP cleanup_test_data tool
   * Removes test projects, resets configuration, handles cleanup safely
   * @param {Object} options - Cleanup options
   * @param {boolean} options.keepProjectsFile - Keep the projects JSON file
   * @param {boolean} options.keepLogs - Keep log files
   * @param {boolean} options.force - Force cleanup without confirmation
   * @param {string} options.mcpServerPath - Path to MCP server
   * @returns {Promise<Object>} - Cleanup result with deleted items and errors
   */
  static async cleanupTestData(options = {}) {
    const mcpServerPath = options.mcpServerPath || './mcp-server';
    
    try {
      const args = {
        keep_projects_file: options.keepProjectsFile || false,
        keep_logs: options.keepLogs || false,
        force: options.force || false
      };
      
      const command = `echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"mcp-server__cleanup_test_data","arguments":${JSON.stringify(args)}},"id":1}' | ${mcpServerPath} --test-mode`;
      
      const { stdout } = await execAsync(command);
      const response = JSON.parse(stdout);
      
      if (response.result && response.result.content) {
        const content = response.result.content[0].text;
        try {
          const result = JSON.parse(content);
          if (result.success) {
            console.log(`Cleanup completed: ${result.message}`);
            return result;
          } else {
            console.warn(`Cleanup completed with errors: ${result.message}`);
            return result;
          }
        } catch (parseError) {
          console.error('Failed to parse cleanup response:', content);
          throw parseError;
        }
      }
      
      throw new Error('No response from cleanup_test_data tool');
    } catch (error) {
      console.error('Error cleaning up test data:', error);
      throw error;
    }
  }

  /**
   * Helper method to get standardized test paths from configuration
   * Provides consistent path resolution across all tests
   * @param {string} mcpServerPath - Path to MCP server
   * @returns {Promise<Object>} - Object with standardized paths
   */
  static async getTestPaths(mcpServerPath = './mcp-server') {
    try {
      const config = await this.getTestConfig({ 
        format: 'summary', 
        mcpServerPath 
      });
      
      if (!config) {
        throw new Error('Could not retrieve test configuration');
      }
      
      return {
        projectsFile: config.projects_file,
        testDataDir: config.test_data_dir,
        baseUrl: config.base_url,
        serverPort: config.server_port || 8080,
        timeout: config.default_timeout || 30000
      };
    } catch (error) {
      console.error('Error getting test paths:', error);
      // Return sensible defaults if config fails
      return {
        projectsFile: '.e2e-projects.json',
        testDataDir: '.e2e-projects',
        baseUrl: 'http://localhost:8080',
        serverPort: 8080,
        timeout: 30000
      };
    }
  }
}

module.exports = TestLifecycle;