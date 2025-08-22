# E2E Test Suite

This test suite uses Playwright with the Page Object Model pattern for better organization and maintainability.

## Structure

```
e2e-tests/e2e/
├── pages/           # Page Object classes
│   ├── DashboardPage.js
│   └── ProjectSidebarPage.js
├── specs/           # Test specifications
│   ├── dashboard.spec.js
│   └── projects.spec.js
├── utils/           # Helper utilities
│   └── TestHelpers.js
├── fixtures/        # Test setup/teardown
│   └── TestFixtures.js
└── data/           # Test data (if needed)
```

## Running Tests

```bash
# Run all tests
make test-browser

# Run with UI
npx playwright test --headed

# Debug mode
npx playwright test --debug

# Run specific test file
npx playwright test e2e-tests/e2e/specs/dashboard.spec.js

# View test report
npx playwright show-report
```

## Key Features

1. **Page Object Model**: Each page has its own class with selectors and methods
2. **Test Isolation**: Each test uses a separate test projects file
3. **Automatic Cleanup**: Test data is reset before and after tests
4. **Configurable**: Uses environment variables for projects file location
5. **Comprehensive**: Tests both UI interactions and HTMX functionality

## Environment Variables

- `PROJECTS_FILE`: Path to projects JSON file (defaults to `projects.json`)
  - Tests use `test_projects.json` to avoid polluting production data

## Adding New Tests

1. Create a new page object in `pages/` if testing a new page
2. Add test specs in `specs/`
3. Use existing helpers from `utils/TestHelpers.js`
4. Follow the existing patterns for consistency