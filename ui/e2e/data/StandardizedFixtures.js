// StandardizedFixtures.js - Pre-configured test fixtures using TestDataFactory
// Provides ready-to-use test scenarios for common testing needs

const TestDataFactory = require('./TestDataFactory');
const fs = require('fs');
const path = require('path');

class StandardizedFixtures {
  /**
   * Empty project setup for new user experience tests
   * @returns {Object} - Empty fixture configuration
   */
  static async setupEmptyFixture() {
    await TestDataFactory.resetTestData();
    
    return {
      projects: [],
      projectsFile: '.e2e-projects.json',
      basePath: '.e2e-projects',
      metadata: {
        fixture: 'empty',
        purpose: 'new-user-experience',
        created_at: new Date().toISOString()
      }
    };
  }

  /**
   * Single project setup for basic functionality tests
   * @param {Object} projectOverrides - Optional project property overrides
   * @returns {Object} - Single project fixture
   */
  static async setupSingleProjectFixture(projectOverrides = {}) {
    const fixture = await TestDataFactory.initializeTestData({
      projectCount: 1,
      includeComponents: true,
      includeLogs: false
    });
    
    // Apply any project overrides
    if (Object.keys(projectOverrides).length > 0) {
      fixture.projects[0] = { ...fixture.projects[0], ...projectOverrides };
      
      // Update projects file with overrides
      fs.writeFileSync(
        fixture.projectsFile,
        JSON.stringify(fixture.projects, null, 2)
      );
    }
    
    return {
      ...fixture,
      project: fixture.projects[0], // Convenience accessor
      metadata: {
        fixture: 'single-project',
        purpose: 'basic-functionality',
        created_at: new Date().toISOString()
      }
    };
  }

  /**
   * Multiple projects setup for project switching and management tests
   * @param {number} count - Number of projects (default: 3)
   * @returns {Object} - Multiple projects fixture
   */
  static async setupMultipleProjectsFixture(count = 3) {
    const fixture = await TestDataFactory.initializeTestData({
      projectCount: count,
      includeComponents: true,
      includeLogs: true
    });
    
    // Set different statuses for variety
    fixture.projects[0].status = 'active';
    if (fixture.projects[1]) fixture.projects[1].status = 'inactive';
    if (fixture.projects[2]) fixture.projects[2].status = 'building';
    
    // Update projects file
    fs.writeFileSync(
      fixture.projectsFile,
      JSON.stringify(fixture.projects, null, 2)
    );
    
    return {
      ...fixture,
      activeProject: fixture.projects[0],
      metadata: {
        fixture: 'multiple-projects',
        purpose: 'project-management',
        project_count: count,
        created_at: new Date().toISOString()
      }
    };
  }

  /**
   * Watch mode testing fixture with file change simulation
   * @returns {Object} - Watch mode fixture
   */
  static async setupWatchModeFixture() {
    const fixture = await this.setupSingleProjectFixture();
    const project = fixture.project;
    
    // Create source files for watch mode
    const srcDir = path.join(project.path, 'src');
    fs.mkdirSync(srcDir, { recursive: true });
    
    // Create main source file
    const mainFile = path.join(srcDir, 'main.rs');
    fs.writeFileSync(mainFile, `fn main() {
    println!("Hello from test project");
}`);
    
    // Create lib file
    const libFile = path.join(srcDir, 'lib.rs');
    fs.writeFileSync(libFile, `pub fn test_function() -> String {
    "test".to_string()
}`);
    
    // Create Cargo.toml for Rust projects
    if (project.language === 'rust') {
      const cargoToml = `[package]
name = "${project.name.toLowerCase().replace(/\s+/g, '-')}"
version = "0.1.0"
edition = "2021"

[dependencies]
`;
      fs.writeFileSync(path.join(project.path, 'Cargo.toml'), cargoToml);
    }
    
    return {
      ...fixture,
      sourceFiles: {
        main: mainFile,
        lib: libFile
      },
      metadata: {
        fixture: 'watch-mode',
        purpose: 'file-watching',
        created_at: new Date().toISOString()
      }
    };
  }

  /**
   * Build testing fixture with various build states
   * @returns {Object} - Build testing fixture
   */
  static async setupBuildTestingFixture() {
    const fixture = await this.setupMultipleProjectsFixture(3);
    
    // Set up different build states
    const buildStates = [
      { status: 'success', stdout: 'Build completed successfully\n', stderr: '' },
      { status: 'failed', stdout: '', stderr: 'Error: Compilation failed\n' },
      { status: 'building', stdout: 'Building...\n', stderr: '' }
    ];
    
    fixture.projects.forEach((project, index) => {
      const buildState = buildStates[index % buildStates.length];
      const buildDir = path.join(project.path, '.ftl', 'build');
      fs.mkdirSync(buildDir, { recursive: true });
      
      // Create build output file
      const buildOutput = TestDataFactory.createCommandOutput({
        command: 'ftl build',
        status: buildState.status,
        stdout: buildState.stdout,
        stderr: buildState.stderr,
        metadata: { project_id: project.id }
      });
      
      fs.writeFileSync(
        path.join(buildDir, 'last-build.json'),
        JSON.stringify(buildOutput, null, 2)
      );
    });
    
    return {
      ...fixture,
      buildStates,
      metadata: {
        fixture: 'build-testing',
        purpose: 'build-operations',
        created_at: new Date().toISOString()
      }
    };
  }

  /**
   * Error handling fixture with various error scenarios
   * @returns {Object} - Error handling fixture
   */
  static async setupErrorHandlingFixture() {
    const fixture = await this.setupSingleProjectFixture();
    const project = fixture.project;
    
    // Create various error scenarios
    const errors = [
      TestDataFactory.createError({
        type: 'build_error',
        message: 'Failed to compile: syntax error',
        metadata: { project_id: project.id }
      }),
      TestDataFactory.createError({
        type: 'runtime_error',
        message: 'Panic: index out of bounds',
        severity: 'critical',
        recoverable: false,
        metadata: { project_id: project.id }
      }),
      TestDataFactory.createError({
        type: 'network_error',
        message: 'Failed to connect to server',
        severity: 'warning',
        recoverable: true,
        metadata: { project_id: project.id }
      })
    ];
    
    // Store errors in project
    const errorsDir = path.join(project.path, '.ftl', 'errors');
    fs.mkdirSync(errorsDir, { recursive: true });
    
    errors.forEach((error, index) => {
      fs.writeFileSync(
        path.join(errorsDir, `error-${index}.json`),
        JSON.stringify(error, null, 2)
      );
    });
    
    return {
      ...fixture,
      errors,
      metadata: {
        fixture: 'error-handling',
        purpose: 'error-scenarios',
        error_count: errors.length,
        created_at: new Date().toISOString()
      }
    };
  }

  /**
   * Performance testing fixture with large datasets
   * @returns {Object} - Performance testing fixture
   */
  static async setupPerformanceFixture() {
    // Create many projects for list performance testing
    const fixture = await TestDataFactory.initializeTestData({
      projectCount: 20,
      includeComponents: false,
      includeLogs: false
    });
    
    // Add lots of log entries to first project
    const project = fixture.projects[0];
    const logsDir = path.join(project.path, '.ftl', 'logs');
    fs.mkdirSync(logsDir, { recursive: true });
    
    // Create 1000 log entries
    const logs = TestDataFactory.createLogEntries(1000, {
      metadata: { project_id: project.id }
    });
    
    // Write in batches to avoid memory issues
    const batchSize = 100;
    for (let i = 0; i < logs.length; i += batchSize) {
      const batch = logs.slice(i, i + batchSize);
      fs.appendFileSync(
        path.join(logsDir, 'performance.log'),
        batch.map(log => JSON.stringify(log)).join('\n') + '\n'
      );
    }
    
    return {
      ...fixture,
      logCount: logs.length,
      metadata: {
        fixture: 'performance',
        purpose: 'performance-testing',
        project_count: 20,
        log_count: 1000,
        created_at: new Date().toISOString()
      }
    };
  }

  /**
   * Console output testing fixture
   * @returns {Object} - Console output fixture
   */
  static async setupConsoleOutputFixture() {
    const fixture = await this.setupSingleProjectFixture();
    const project = fixture.project;
    
    // Create various command outputs
    const outputs = [
      TestDataFactory.createCommandOutput({
        command: 'ftl build',
        stdout: 'Compiling project...\n✓ Build successful',
        metadata: { project_id: project.id }
      }),
      TestDataFactory.createCommandOutput({
        command: 'ftl up',
        stdout: 'Starting server on port 8080...\nServer ready',
        metadata: { project_id: project.id }
      }),
      TestDataFactory.createCommandOutput({
        command: 'ftl watch',
        stdout: 'Watching for file changes...\n',
        metadata: { project_id: project.id }
      })
    ];
    
    // Store outputs
    const outputDir = path.join(project.path, '.ftl', 'output');
    fs.mkdirSync(outputDir, { recursive: true });
    
    outputs.forEach((output, index) => {
      fs.writeFileSync(
        path.join(outputDir, `output-${index}.json`),
        JSON.stringify(output, null, 2)
      );
    });
    
    return {
      ...fixture,
      outputs,
      metadata: {
        fixture: 'console-output',
        purpose: 'output-testing',
        output_count: outputs.length,
        created_at: new Date().toISOString()
      }
    };
  }

  /**
   * Clean up all test fixtures
   */
  static async cleanup() {
    await TestDataFactory.resetTestData();
    
    // Also clean up any additional test files
    const additionalFiles = [
      '.e2e-projects.json',
      'test_projects.json'
    ];
    
    for (const file of additionalFiles) {
      if (fs.existsSync(file)) {
        fs.unlinkSync(file);
        console.log(`Cleaned up: ${file}`);
      }
    }
  }

  /**
   * Get fixture by name
   * @param {string} name - Fixture name
   * @param {Object} options - Fixture options
   * @returns {Object} - Fixture data
   */
  static async getFixture(name, options = {}) {
    const fixtures = {
      'empty': () => this.setupEmptyFixture(),
      'single': () => this.setupSingleProjectFixture(options.projectOverrides),
      'multiple': () => this.setupMultipleProjectsFixture(options.count),
      'watch': () => this.setupWatchModeFixture(),
      'build': () => this.setupBuildTestingFixture(),
      'error': () => this.setupErrorHandlingFixture(),
      'performance': () => this.setupPerformanceFixture(),
      'console': () => this.setupConsoleOutputFixture()
    };
    
    const fixtureSetup = fixtures[name];
    if (!fixtureSetup) {
      throw new Error(`Unknown fixture: ${name}`);
    }
    
    return await fixtureSetup();
  }
}

module.exports = StandardizedFixtures;