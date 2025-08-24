// TestDataManager.js - Centralized test data management
// Coordinates test data lifecycle and provides utilities for tests

const TestDataFactory = require('./TestDataFactory');
const StandardizedFixtures = require('./StandardizedFixtures');
const fs = require('fs');
const path = require('path');

class TestDataManager {
  constructor() {
    this.currentFixture = null;
    this.testRunId = TestDataFactory.generateId('run');
    this.dataSnapshots = [];
  }

  /**
   * Initialize test data for a test suite
   * @param {string} fixtureName - Name of the fixture to use
   * @param {Object} options - Fixture options
   * @returns {Object} - Initialized fixture data
   */
  async initialize(fixtureName = 'single', options = {}) {
    // Clean up any existing data
    await this.cleanup();
    
    // Set test run ID in environment
    process.env.TEST_RUN_ID = this.testRunId;
    
    // Load the requested fixture
    this.currentFixture = await StandardizedFixtures.getFixture(fixtureName, options);
    
    // Take initial snapshot
    this.takeSnapshot('initial');
    
    console.log(`Test data initialized with fixture: ${fixtureName}`);
    return this.currentFixture;
  }

  /**
   * Get current fixture data
   * @returns {Object} - Current fixture
   */
  getCurrentFixture() {
    if (!this.currentFixture) {
      throw new Error('No fixture initialized. Call initialize() first.');
    }
    return this.currentFixture;
  }

  /**
   * Get a specific project from current fixture
   * @param {number|string} identifier - Project index or ID
   * @returns {Object} - Project data
   */
  getProject(identifier = 0) {
    const fixture = this.getCurrentFixture();
    
    if (typeof identifier === 'number') {
      return fixture.projects[identifier];
    }
    
    return fixture.projects.find(p => p.id === identifier || p.name === identifier);
  }

  /**
   * Update project data
   * @param {string} projectId - Project ID
   * @param {Object} updates - Updates to apply
   * @returns {Object} - Updated project
   */
  async updateProject(projectId, updates) {
    const fixture = this.getCurrentFixture();
    const projectIndex = fixture.projects.findIndex(p => p.id === projectId);
    
    if (projectIndex === -1) {
      throw new Error(`Project not found: ${projectId}`);
    }
    
    // Update project
    fixture.projects[projectIndex] = {
      ...fixture.projects[projectIndex],
      ...updates,
      updated_at: new Date().toISOString()
    };
    
    // Persist to file
    await this.persistProjects();
    
    return fixture.projects[projectIndex];
  }

  /**
   * Add a new project to current fixture
   * @param {Object} projectData - Project data or overrides
   * @returns {Object} - Created project
   */
  async addProject(projectData = {}) {
    const fixture = this.getCurrentFixture();
    
    // Create new project
    const project = TestDataFactory.createProject(projectData);
    
    // Add to fixture
    fixture.projects.push(project);
    
    // Create project directory
    fs.mkdirSync(project.path, { recursive: true });
    
    // Create ftl.toml
    const ftlConfig = `name = "${project.name}"
language = "${project.language}"
`;
    fs.writeFileSync(path.join(project.path, 'ftl.toml'), ftlConfig);
    
    // Persist to file
    await this.persistProjects();
    
    return project;
  }

  /**
   * Remove a project from current fixture
   * @param {string} projectId - Project ID to remove
   */
  async removeProject(projectId) {
    const fixture = this.getCurrentFixture();
    const projectIndex = fixture.projects.findIndex(p => p.id === projectId);
    
    if (projectIndex === -1) {
      throw new Error(`Project not found: ${projectId}`);
    }
    
    const project = fixture.projects[projectIndex];
    
    // Remove from array
    fixture.projects.splice(projectIndex, 1);
    
    // Remove project directory
    if (fs.existsSync(project.path)) {
      fs.rmSync(project.path, { recursive: true, force: true });
    }
    
    // Persist to file
    await this.persistProjects();
  }

  /**
   * Create command output for a project
   * @param {string} projectId - Project ID
   * @param {Object} outputData - Output data or overrides
   * @returns {Object} - Created output
   */
  async createOutput(projectId, outputData = {}) {
    const project = this.getProject(projectId);
    
    const output = TestDataFactory.createCommandOutput({
      ...outputData,
      metadata: {
        ...outputData.metadata,
        project_id: project.id
      }
    });
    
    // Store output in project
    const outputDir = path.join(project.path, '.ftl', 'output');
    fs.mkdirSync(outputDir, { recursive: true });
    
    fs.writeFileSync(
      path.join(outputDir, `${output.id}.json`),
      JSON.stringify(output, null, 2)
    );
    
    return output;
  }

  /**
   * Create log entries for a project
   * @param {string} projectId - Project ID
   * @param {number} count - Number of log entries
   * @param {Object} baseOverrides - Base overrides for all entries
   * @returns {Array} - Created log entries
   */
  async createLogs(projectId, count = 5, baseOverrides = {}) {
    const project = this.getProject(projectId);
    
    const logs = TestDataFactory.createLogEntries(count, {
      ...baseOverrides,
      metadata: {
        ...baseOverrides.metadata,
        project_id: project.id
      }
    });
    
    // Store logs in project
    const logsDir = path.join(project.path, '.ftl', 'logs');
    fs.mkdirSync(logsDir, { recursive: true });
    
    const logFile = path.join(logsDir, `${TestDataFactory.generateId('log')}.jsonl`);
    fs.writeFileSync(
      logFile,
      logs.map(log => JSON.stringify(log)).join('\n')
    );
    
    return logs;
  }

  /**
   * Simulate file change for watch mode testing
   * @param {string} projectId - Project ID
   * @param {string} filePath - Relative file path
   * @param {string} content - New file content
   * @returns {Object} - Watch event data
   */
  async simulateFileChange(projectId, filePath, content) {
    const project = this.getProject(projectId);
    const fullPath = path.join(project.path, filePath);
    
    // Ensure directory exists
    fs.mkdirSync(path.dirname(fullPath), { recursive: true });
    
    // Write file
    fs.writeFileSync(fullPath, content);
    
    // Create watch event
    const watchEvent = TestDataFactory.createWatchEvent({
      path: filePath,
      type: 'file_changed',
      metadata: { project_id: project.id }
    });
    
    // Store watch event
    const eventsDir = path.join(project.path, '.ftl', 'events');
    fs.mkdirSync(eventsDir, { recursive: true });
    
    fs.writeFileSync(
      path.join(eventsDir, `${watchEvent.id}.json`),
      JSON.stringify(watchEvent, null, 2)
    );
    
    return watchEvent;
  }

  /**
   * Take a snapshot of current test data state
   * @param {string} name - Snapshot name
   * @returns {Object} - Snapshot data
   */
  takeSnapshot(name) {
    const snapshot = {
      name,
      timestamp: new Date().toISOString(),
      fixture: this.currentFixture ? { ...this.currentFixture } : null,
      testRunId: this.testRunId
    };
    
    this.dataSnapshots.push(snapshot);
    return snapshot;
  }

  /**
   * Get all snapshots
   * @returns {Array} - All snapshots
   */
  getSnapshots() {
    return this.dataSnapshots;
  }

  /**
   * Restore from a snapshot
   * @param {string} name - Snapshot name
   */
  async restoreSnapshot(name) {
    const snapshot = this.dataSnapshots.find(s => s.name === name);
    
    if (!snapshot) {
      throw new Error(`Snapshot not found: ${name}`);
    }
    
    // Clean current data
    await this.cleanup();
    
    // Restore fixture
    this.currentFixture = snapshot.fixture;
    
    // Recreate file system structure
    if (this.currentFixture) {
      await this.recreateFileStructure();
    }
  }

  /**
   * Persist projects to file
   */
  async persistProjects() {
    const fixture = this.getCurrentFixture();
    
    fs.writeFileSync(
      fixture.projectsFile,
      JSON.stringify(fixture.projects, null, 2)
    );
  }

  /**
   * Recreate file structure from current fixture
   */
  async recreateFileStructure() {
    const fixture = this.getCurrentFixture();
    
    // Ensure base directory exists
    fs.mkdirSync(fixture.basePath, { recursive: true });
    
    // Recreate each project directory
    for (const project of fixture.projects) {
      fs.mkdirSync(project.path, { recursive: true });
      
      // Create ftl.toml
      const ftlConfig = `name = "${project.name}"
language = "${project.language}"
`;
      fs.writeFileSync(path.join(project.path, 'ftl.toml'), ftlConfig);
    }
    
    // Write projects file
    await this.persistProjects();
  }

  /**
   * Validate current test data state
   * @returns {Object} - Validation result
   */
  async validate() {
    const result = {
      valid: true,
      errors: [],
      warnings: []
    };
    
    if (!this.currentFixture) {
      result.valid = false;
      result.errors.push('No fixture initialized');
      return result;
    }
    
    const fixture = this.currentFixture;
    
    // Check projects file exists
    if (!fs.existsSync(fixture.projectsFile)) {
      result.errors.push(`Projects file missing: ${fixture.projectsFile}`);
      result.valid = false;
    }
    
    // Check each project directory
    for (const project of fixture.projects) {
      if (!fs.existsSync(project.path)) {
        result.warnings.push(`Project directory missing: ${project.path}`);
      } else {
        // Check for ftl.toml
        const ftlPath = path.join(project.path, 'ftl.toml');
        if (!fs.existsSync(ftlPath)) {
          result.warnings.push(`FTL config missing: ${ftlPath}`);
        }
      }
    }
    
    return result;
  }

  /**
   * Get test data statistics
   * @returns {Object} - Statistics
   */
  getStatistics() {
    const fixture = this.getCurrentFixture();
    
    return {
      testRunId: this.testRunId,
      fixtureName: fixture?.metadata?.fixture || 'unknown',
      projectCount: fixture?.projects?.length || 0,
      snapshotCount: this.dataSnapshots.length,
      diskUsage: this.calculateDiskUsage(),
      createdAt: fixture?.metadata?.created_at,
      currentTime: new Date().toISOString()
    };
  }

  /**
   * Calculate disk usage of test data
   * @returns {number} - Size in bytes
   */
  calculateDiskUsage() {
    const fixture = this.getCurrentFixture();
    if (!fixture) return 0;
    
    let totalSize = 0;
    
    const calculateDirSize = (dirPath) => {
      if (!fs.existsSync(dirPath)) return 0;
      
      let size = 0;
      const entries = fs.readdirSync(dirPath, { withFileTypes: true });
      
      for (const entry of entries) {
        const fullPath = path.join(dirPath, entry.name);
        if (entry.isDirectory()) {
          size += calculateDirSize(fullPath);
        } else {
          size += fs.statSync(fullPath).size;
        }
      }
      
      return size;
    };
    
    // Calculate size of base directory
    totalSize += calculateDirSize(fixture.basePath);
    
    // Add size of projects file
    if (fs.existsSync(fixture.projectsFile)) {
      totalSize += fs.statSync(fixture.projectsFile).size;
    }
    
    return totalSize;
  }

  /**
   * Clean up all test data
   */
  async cleanup() {
    await StandardizedFixtures.cleanup();
    this.currentFixture = null;
    this.dataSnapshots = [];
    
    console.log(`Test data cleaned up (run: ${this.testRunId})`);
  }
}

// Export singleton instance
module.exports = new TestDataManager();