# Test Data Standardization Migration Guide

## Overview

Phase 6 of the professional testing plan introduces a standardized test data system that replaces ad-hoc test data creation with a consistent, factory-based approach.

## New Components

### 1. TestDataFactory (`data/TestDataFactory.js`)
- Provides factory methods for creating all test data types
- Generates unique IDs with prefixes
- Creates consistent data structures
- Handles batch data creation

### 2. StandardizedFixtures (`data/StandardizedFixtures.js`)
- Pre-configured test scenarios
- Ready-to-use fixtures for common test cases
- Includes: empty, single, multiple, watch, build, error, performance, console

### 3. TestDataManager (`data/TestDataManager.js`)
- Centralized test data lifecycle management
- Singleton instance for coordinating test data
- Snapshot/restore capabilities
- Validation and statistics

## Migration Steps

### Step 1: Update Test Imports

**Before:**
```javascript
const TestHelpers = require('../utils/TestHelpers');
const fs = require('fs');
const path = require('path');
```

**After:**
```javascript
const TestHelpers = require('../utils/TestHelpers');
const TestDataManager = require('../data/TestDataManager');
const TestDataFactory = require('../data/TestDataFactory');
```

### Step 2: Replace Manual Project Creation

**Before:**
```javascript
// Manual project creation
const projectName = `test-project-${Date.now()}`;
const projectPath = path.join('.e2e-projects', projectName);
const testProject = {
    name: projectName,
    path: projectPath,
    added_at: new Date().toISOString()
};
fs.writeFileSync('.e2e-projects.json', JSON.stringify([testProject]));
```

**After:**
```javascript
// Use standardized fixture
const fixture = await TestDataManager.initialize('single');
const project = fixture.project;
```

### Step 3: Update Test Setup

**Before:**
```javascript
test.beforeEach(async ({ page }) => {
    await TestHelpers.resetTestProjectsFile();
    // Manual setup...
});
```

**After:**
```javascript
test.beforeEach(async ({ page }) => {
    // Choose appropriate fixture
    await TestDataManager.initialize('single'); // or 'multiple', 'watch', etc.
});
```

### Step 4: Update Test Cleanup

**Before:**
```javascript
test.afterEach(async () => {
    if (fs.existsSync('test_projects.json')) {
        fs.unlinkSync('test_projects.json');
    }
    // Manual cleanup...
});
```

**After:**
```javascript
test.afterEach(async () => {
    await TestDataManager.cleanup();
});
```

## Fixture Selection Guide

Choose the appropriate fixture based on your test requirements:

| Fixture | Use Case | Projects | Additional Data |
|---------|----------|----------|-----------------|
| `empty` | New user experience tests | 0 | None |
| `single` | Basic functionality tests | 1 | Components |
| `multiple` | Project switching tests | 3+ | Components, Logs |
| `watch` | File watching tests | 1 | Source files |
| `build` | Build operation tests | 3 | Build outputs |
| `error` | Error handling tests | 1 | Error scenarios |
| `performance` | Performance tests | 20 | 1000+ logs |
| `console` | Console output tests | 1 | Command outputs |

## Common Migration Patterns

### Pattern 1: Dynamic Project Creation

**Before:**
```javascript
const projects = [];
for (let i = 0; i < 3; i++) {
    projects.push({
        name: `Project ${i}`,
        path: `./test-${i}`
    });
}
```

**After:**
```javascript
const fixture = await TestDataManager.initialize('multiple', { count: 3 });
const projects = fixture.projects;
```

### Pattern 2: Adding Test Data During Test

**Before:**
```javascript
// Manual file creation
fs.writeFileSync(path.join(projectPath, 'test.log'), 'log data');
```

**After:**
```javascript
// Use manager methods
await TestDataManager.createLogs(project.id, 5);
await TestDataManager.createOutput(project.id, { stdout: 'test output' });
```

### Pattern 3: File Change Simulation

**Before:**
```javascript
// Manual file modification
fs.writeFileSync('./src/main.rs', 'new content');
// Wait for watch to detect...
```

**After:**
```javascript
// Structured file change simulation
const watchEvent = await TestDataManager.simulateFileChange(
    project.id,
    'src/main.rs',
    'new content'
);
```

## Advanced Features

### Snapshots

Take snapshots during test for debugging:
```javascript
TestDataManager.takeSnapshot('before-action');
// Perform test actions...
TestDataManager.takeSnapshot('after-action');

// Restore if needed
await TestDataManager.restoreSnapshot('before-action');
```

### Validation

Validate test data integrity:
```javascript
const validation = await TestDataManager.validate();
if (!validation.valid) {
    console.error('Test data issues:', validation.errors);
}
```

### Statistics

Get test data statistics:
```javascript
const stats = TestDataManager.getStatistics();
console.log(`Projects: ${stats.projectCount}`);
console.log(`Disk usage: ${stats.diskUsage} bytes`);
```

### Custom Data

Create custom data using the factory:
```javascript
const customProject = TestDataFactory.createProject({
    name: 'Custom Project',
    language: 'python',
    metadata: { custom: true }
});

const logs = TestDataFactory.createLogEntries(100);
const output = TestDataFactory.createCommandOutput({
    command: 'custom-command',
    exit_code: 42
});
```

## Benefits of Migration

1. **Consistency**: All tests use the same data structures
2. **Maintainability**: Centralized data management
3. **Reusability**: Pre-configured fixtures for common scenarios
4. **Debugging**: Snapshots and validation help identify issues
5. **Performance**: Optimized data creation and cleanup
6. **Isolation**: Each test gets fresh, isolated data
7. **Flexibility**: Easy to extend with new fixtures and data types

## Troubleshooting

### Issue: "No fixture initialized"
**Solution:** Ensure `TestDataManager.initialize()` is called before accessing data

### Issue: Projects not appearing in UI
**Solution:** Check that the fixture creates the correct projects file format

### Issue: Cleanup not working
**Solution:** Ensure `TestDataManager.cleanup()` is called in afterEach hooks

### Issue: Custom data not persisting
**Solution:** Use `TestDataManager.persistProjects()` after modifications

## Example Migration

See `specs/example-standardized-data.spec.js` for complete examples of:
- All fixture types
- Dynamic data creation
- Snapshot/restore
- Validation and statistics
- Factory usage

## Checklist for Migration

- [ ] Update imports to include TestDataManager and TestDataFactory
- [ ] Replace manual project creation with fixture initialization
- [ ] Update beforeEach hooks to use TestDataManager.initialize()
- [ ] Update afterEach hooks to use TestDataManager.cleanup()
- [ ] Replace file system operations with TestDataManager methods
- [ ] Choose appropriate fixtures for each test
- [ ] Add validation checks where appropriate
- [ ] Use snapshots for complex test debugging
- [ ] Review and update any custom data creation

## Next Steps

After migrating existing tests:
1. Remove old test data helper functions
2. Update CI/CD scripts to use new cleanup methods
3. Add fixture-specific tests for new scenarios
4. Document any custom fixtures created
5. Monitor test performance improvements