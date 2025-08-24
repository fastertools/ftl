// TestDataFactory.js - Factory for creating standardized test data
// Provides consistent, predictable test data across all e2e tests

const path = require('path');
const fs = require('fs');
const crypto = require('crypto');

class TestDataFactory {
  /**
   * Generate a unique identifier for test data
   * @param {string} prefix - Optional prefix for the ID
   * @returns {string} - Unique identifier
   */
  static generateId(prefix = '') {
    const timestamp = Date.now();
    const random = crypto.randomBytes(4).toString('hex');
    return prefix ? `${prefix}-${timestamp}-${random}` : `${timestamp}-${random}`;
  }

  /**
   * Create a standardized project configuration
   * @param {Object} overrides - Optional overrides for project properties
   * @returns {Object} - Project configuration object
   */
  static createProject(overrides = {}) {
    const projectId = this.generateId('project');
    const now = new Date().toISOString();
    
    const defaults = {
      id: projectId,
      name: `Test Project ${projectId}`,
      path: path.join('.e2e-projects', projectId),
      type: 'rust', // Default project type
      language: 'rust',
      description: 'E2E test project',
      created_at: now,
      updated_at: now,
      last_active: now,
      status: 'inactive',
      metadata: {
        test_id: projectId,
        test_run: process.env.TEST_RUN_ID || 'local',
        created_by: 'e2e-test'
      }
    };
    
    return { ...defaults, ...overrides };
  }

  /**
   * Create a batch of projects for testing lists and filtering
   * @param {number} count - Number of projects to create
   * @param {Object} baseOverrides - Overrides applied to all projects
   * @returns {Array} - Array of project configurations
   */
  static createProjects(count = 3, baseOverrides = {}) {
    const projects = [];
    const languages = ['rust', 'python', 'go'];
    const statuses = ['active', 'inactive', 'building'];
    
    for (let i = 0; i < count; i++) {
      projects.push(this.createProject({
        ...baseOverrides,
        name: `Test Project ${i + 1}`,
        language: languages[i % languages.length],
        status: statuses[i % statuses.length],
        metadata: {
          ...baseOverrides.metadata,
          index: i,
          batch_id: this.generateId('batch')
        }
      }));
    }
    
    return projects;
  }

  /**
   * Create a component configuration for FTL projects
   * @param {Object} overrides - Optional overrides
   * @returns {Object} - Component configuration
   */
  static createComponent(overrides = {}) {
    const componentId = this.generateId('component');
    
    const defaults = {
      id: componentId,
      name: `test-component-${componentId}`,
      type: 'tool',
      path: `./components/${componentId}`,
      language: 'rust',
      version: '0.1.0',
      dependencies: [],
      build_status: 'not_built',
      last_build: null,
      metadata: {
        test_id: componentId,
        created_by: 'e2e-test'
      }
    };
    
    return { ...defaults, ...overrides };
  }

  /**
   * Create a build configuration
   * @param {Object} overrides - Optional overrides
   * @returns {Object} - Build configuration
   */
  static createBuildConfig(overrides = {}) {
    const defaults = {
      id: this.generateId('build'),
      project_id: null, // Should be set by caller
      target: 'wasm32-wasi',
      profile: 'debug',
      features: [],
      environment: {
        CARGO_TARGET_DIR: './target',
        RUST_LOG: 'info'
      },
      options: {
        verbose: false,
        parallel: true,
        cache: true
      }
    };
    
    return { ...defaults, ...overrides };
  }

  /**
   * Create command output data for testing console output
   * @param {Object} overrides - Optional overrides
   * @returns {Object} - Command output data
   */
  static createCommandOutput(overrides = {}) {
    const defaults = {
      id: this.generateId('output'),
      command: 'ftl build',
      timestamp: new Date().toISOString(),
      status: 'success',
      exit_code: 0,
      stdout: 'Build completed successfully\n',
      stderr: '',
      duration_ms: 1500,
      metadata: {
        project_id: null, // Should be set by caller
        component_id: null
      }
    };
    
    return { ...defaults, ...overrides };
  }

  /**
   * Create log entry data
   * @param {Object} overrides - Optional overrides
   * @returns {Object} - Log entry
   */
  static createLogEntry(overrides = {}) {
    const defaults = {
      id: this.generateId('log'),
      timestamp: new Date().toISOString(),
      level: 'info',
      message: 'Test log message',
      source: 'test-component',
      metadata: {
        project_id: null,
        component_id: null,
        request_id: this.generateId('req')
      }
    };
    
    return { ...defaults, ...overrides };
  }

  /**
   * Create a collection of log entries with different levels
   * @param {number} count - Number of log entries
   * @param {Object} baseOverrides - Base overrides for all entries
   * @returns {Array} - Array of log entries
   */
  static createLogEntries(count = 5, baseOverrides = {}) {
    const levels = ['debug', 'info', 'warn', 'error'];
    const messages = [
      'Starting component initialization',
      'Processing request',
      'Connection established',
      'Warning: High memory usage',
      'Error: Failed to connect to database'
    ];
    
    const entries = [];
    const baseTime = Date.now();
    
    for (let i = 0; i < count; i++) {
      entries.push(this.createLogEntry({
        ...baseOverrides,
        timestamp: new Date(baseTime + i * 1000).toISOString(),
        level: levels[i % levels.length],
        message: messages[i % messages.length],
        metadata: {
          ...baseOverrides.metadata,
          sequence: i
        }
      }));
    }
    
    return entries;
  }

  /**
   * Create watch mode event data
   * @param {Object} overrides - Optional overrides
   * @returns {Object} - Watch event
   */
  static createWatchEvent(overrides = {}) {
    const defaults = {
      id: this.generateId('watch'),
      timestamp: new Date().toISOString(),
      type: 'file_changed',
      path: './src/main.rs',
      action: 'rebuild',
      status: 'pending',
      metadata: {
        project_id: null,
        previous_build_id: null
      }
    };
    
    return { ...defaults, ...overrides };
  }

  /**
   * Create port allocation data
   * @param {Object} overrides - Optional overrides
   * @returns {Object} - Port allocation
   */
  static createPortAllocation(overrides = {}) {
    const defaults = {
      port: 8080 + Math.floor(Math.random() * 1000),
      service: 'ftl-console',
      allocated_at: new Date().toISOString(),
      pid: process.pid,
      status: 'allocated'
    };
    
    return { ...defaults, ...overrides };
  }

  /**
   * Create error data for testing error handling
   * @param {Object} overrides - Optional overrides
   * @returns {Object} - Error data
   */
  static createError(overrides = {}) {
    const defaults = {
      id: this.generateId('error'),
      timestamp: new Date().toISOString(),
      type: 'build_error',
      message: 'Failed to compile module',
      stack_trace: 'Error: Failed to compile module\n    at build.rs:42:15',
      severity: 'error',
      recoverable: true,
      metadata: {
        project_id: null,
        component_id: null,
        file: './src/main.rs',
        line: 42,
        column: 15
      }
    };
    
    return { ...defaults, ...overrides };
  }

  /**
   * Create test session data
   * @param {Object} overrides - Optional overrides
   * @returns {Object} - Test session
   */
  static createTestSession(overrides = {}) {
    const sessionId = this.generateId('session');
    const now = new Date().toISOString();
    
    const defaults = {
      id: sessionId,
      started_at: now,
      ended_at: null,
      status: 'running',
      test_name: 'e2e-test',
      browser: 'chromium',
      viewport: { width: 1280, height: 720 },
      metadata: {
        test_file: 'unknown.spec.js',
        test_run_id: process.env.TEST_RUN_ID || 'local',
        ci: process.env.CI === 'true'
      }
    };
    
    return { ...defaults, ...overrides };
  }

  /**
   * Create HTMX update event data
   * @param {Object} overrides - Optional overrides
   * @returns {Object} - HTMX event
   */
  static createHTMXEvent(overrides = {}) {
    const defaults = {
      id: this.generateId('htmx'),
      timestamp: new Date().toISOString(),
      trigger: 'poll',
      target: '#console-output',
      swap: 'innerHTML',
      status: 'pending',
      response_time_ms: null,
      metadata: {
        project_id: null,
        poll_interval: 1000
      }
    };
    
    return { ...defaults, ...overrides };
  }

  /**
   * Reset all test data (cleanup utility)
   * @param {string} basePath - Base path for test data
   */
  static async resetTestData(basePath = '.e2e-projects') {
    // Remove existing test data directory
    if (fs.existsSync(basePath)) {
      fs.rmSync(basePath, { recursive: true, force: true });
    }
    
    // Create fresh directory
    fs.mkdirSync(basePath, { recursive: true });
    
    // Create empty projects file
    const projectsFile = '.e2e-projects.json';
    fs.writeFileSync(projectsFile, JSON.stringify([], null, 2));
    
    console.log(`Test data reset: ${basePath} and ${projectsFile}`);
  }

  /**
   * Initialize test data directory with sample data
   * @param {Object} options - Initialization options
   * @returns {Object} - Created test data summary
   */
  static async initializeTestData(options = {}) {
    const {
      projectCount = 1,
      includeComponents = false,
      includeLogs = false,
      basePath = '.e2e-projects'
    } = options;
    
    // Reset first
    await this.resetTestData(basePath);
    
    // Create projects
    const projects = this.createProjects(projectCount);
    
    // Write projects file
    const projectsFile = '.e2e-projects.json';
    fs.writeFileSync(projectsFile, JSON.stringify(projects, null, 2));
    
    // Create project directories
    for (const project of projects) {
      const projectPath = project.path;
      fs.mkdirSync(projectPath, { recursive: true });
      
      // Create ftl.toml
      const ftlConfig = `name = "${project.name}"
language = "${project.language}"

[build]
target = "wasm32-wasi"
`;
      fs.writeFileSync(path.join(projectPath, 'ftl.toml'), ftlConfig);
      
      // Create components if requested
      if (includeComponents) {
        const componentsDir = path.join(projectPath, 'components');
        fs.mkdirSync(componentsDir, { recursive: true });
        
        const component = this.createComponent({
          metadata: { project_id: project.id }
        });
        
        const componentPath = path.join(componentsDir, component.name);
        fs.mkdirSync(componentPath, { recursive: true });
        
        // Create component manifest
        const componentManifest = {
          name: component.name,
          version: component.version,
          type: component.type
        };
        fs.writeFileSync(
          path.join(componentPath, 'component.json'),
          JSON.stringify(componentManifest, null, 2)
        );
      }
      
      // Create logs directory if requested
      if (includeLogs) {
        const logsDir = path.join(projectPath, '.ftl', 'logs');
        fs.mkdirSync(logsDir, { recursive: true });
        
        const logs = this.createLogEntries(5, {
          metadata: { project_id: project.id }
        });
        
        fs.writeFileSync(
          path.join(logsDir, 'test.log'),
          logs.map(log => JSON.stringify(log)).join('\n')
        );
      }
    }
    
    return {
      projects,
      projectsFile,
      basePath
    };
  }
}

module.exports = TestDataFactory;