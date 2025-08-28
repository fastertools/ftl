# Button Visual Regression Testing

This directory contains visual regression tests for the Button Refactoring project. These tests use Playwright's screenshot comparison capabilities to ensure visual consistency across different states, browsers, and viewport sizes.

## Overview

Visual regression tests complement the functional specification and integration tests by validating the actual visual appearance of components. They help catch:

- Unintended styling changes
- Cross-browser rendering differences
- Responsive design issues
- Animation and transition problems
- Color and typography inconsistencies

## Test Structure

### button-visual-regression.spec.js
Main visual regression test suite covering:

- **BaseButton Visual Validation**: Normal, hover, focus, and disabled states
- **SidebarProjectButton Visual Validation**: Active, inactive, different status types
- **Multi-Component Layout Validation**: Complete sidebar and control center layouts
- **Cross-Browser Visual Consistency**: Browser-specific rendering validation
- **Responsive Visual Validation**: Desktop, tablet, and mobile viewports
- **State Transition Visual Validation**: Hover transitions and state changes
- **Error State Visual Validation**: Disabled, error, and loading states

## Screenshot Management

### Baseline Screenshots
First-time test runs create baseline screenshots in:
```
ui/e2e/test-results/button-visual-regression-spec-*/
```

### Screenshot Naming Convention
- `base-button-[type]-[state].png` - BaseButton screenshots
- `sidebar-project-button-[state].png` - SidebarProjectButton screenshots
- `[component]-[browser].png` - Browser-specific screenshots
- `[component]-[resolution].png` - Resolution-specific screenshots

### Updating Baselines
When intentional visual changes are made:
```bash
npx playwright test button-visual-regression --update-snapshots
```

## Configuration

### Visual Test Options
- **Threshold**: 10% difference tolerance
- **Animations**: Disabled for consistent screenshots
- **Full Page**: Used for complete interface layouts
- **Clip**: Used for component-specific screenshots

### Browser Testing
Tests run across:
- Chromium
- Firefox
- WebKit (Safari)

### Viewport Sizes
- Desktop: 1920x1080
- Tablet: 1024x768
- Mobile: 375x667

## Running Visual Tests

### All Visual Tests
```bash
npx playwright test visual/
```

### Specific Visual Test
```bash
npx playwright test button-visual-regression
```

### Update Screenshots
```bash
npx playwright test visual/ --update-snapshots
```

### Debug Mode
```bash
npx playwright test visual/ --debug
```

## Screenshot Analysis

### When Tests Fail
1. Review the diff images in test results
2. Check if changes are intentional
3. Update baselines if changes are correct
4. Fix styling issues if changes are unintended

### Diff Images Location
```
ui/e2e/test-results/button-visual-regression-spec-*/
├── [test-name]-actual.png
├── [test-name]-expected.png
└── [test-name]-diff.png
```

## Best Practices

### Creating Visual Tests
1. **Disable animations** for consistent screenshots
2. **Use specific selectors** for component isolation
3. **Set consistent viewport sizes** for responsive tests
4. **Include meaningful test names** for easy identification

### Maintaining Visual Tests
1. **Review diffs carefully** before updating baselines
2. **Test across all browsers** before committing changes
3. **Keep baselines in version control** for team consistency
4. **Document intentional visual changes** in commit messages

### Troubleshooting
1. **Flaky screenshots**: Check for animations, timing issues, fonts
2. **Cross-browser differences**: Review browser-specific CSS handling
3. **Large diffs**: Verify viewport sizes and element positioning

## Integration with CI/CD

Visual tests should be run in CI/CD pipelines to catch visual regressions early. Consider:

- Running visual tests on pull requests
- Storing baseline screenshots in version control
- Using headless browsers for consistent rendering
- Setting up visual diff reporting in CI

## Performance Considerations

Visual tests are slower than functional tests due to screenshot generation:

- **Selective running**: Use focused test execution during development
- **Parallel execution**: Run visual tests in parallel when possible
- **Optimized screenshots**: Use clipping to reduce screenshot size
- **Staged testing**: Run quick functional tests first, visual tests later

## Maintenance Schedule

- **Weekly**: Review visual test results and address flaky tests
- **Per release**: Update baselines for intentional visual changes
- **Quarterly**: Review and optimize visual test coverage