# Releasing the FTL Python SDK

This guide explains how to release a new version of the FTL Python SDK to PyPI.

## Prerequisites

- You must have write access to the repository
- You must be on the `main` branch with a clean working directory
- All tests must be passing
- PyPI trusted publisher must be configured (see Setup section)

## Setup (One-time)

### 1. Configure PyPI Trusted Publisher

1. Go to [PyPI.org](https://pypi.org) and log in
2. Go to your account settings → Publishing
3. Add a new pending publisher:
   - PyPI Project Name: `ftl-sdk`
   - Owner: `fastertools`
   - Repository name: `ftl-cli`
   - Workflow name: `release-python-sdk.yml`
   - Environment name: `pypi`

4. Repeat for TestPyPI at [test.pypi.org](https://test.pypi.org):
   - Use environment name: `testpypi`

### 2. Configure GitHub Environments

1. Go to repository Settings → Environments
2. Create `pypi` environment:
   - Add protection rule: Required reviewers
   - Add yourself as a reviewer
3. Create `testpypi` environment:
   - Add protection rule: Required reviewers (optional for test)

## Release Process

### 1. Prepare the Release

1. Review changes since the last release:
   ```bash
   git log --oneline sdk/python/v0.1.0..HEAD -- sdk/python/
   ```

2. Update the CHANGELOG.md with specific changes for this release

3. Ensure all tests pass locally:
   ```bash
   cd sdk/python
   tox
   ```

### 2. Create the Release

1. Go to the [Actions tab](https://github.com/fastertools/ftl-cli/actions) in GitHub

2. Click on "Release Python SDK" workflow

3. Click "Run workflow"

4. Fill in the parameters:
   - **Version**: Enter version number (e.g., `0.2.0`)
     - Do NOT include the `v` prefix
     - Follow semantic versioning
   - **Publish to TestPyPI first?**: Recommended for first releases
   - **Is this a pre-release?**: Check for alpha/beta releases

5. Click "Run workflow"

### 3. Approve Deployments

The workflow will pause at two points for manual approval:

1. **TestPyPI deployment** (if enabled):
   - Review the test results
   - Click "Review deployments"
   - Approve the testpypi environment

2. **PyPI deployment**:
   - Verify TestPyPI installation worked
   - Approve the pypi environment

### 4. Monitor the Release

The workflow will:
- Validate version format and progression
- Run full test suite with tox
- Update version in pyproject.toml
- Build sdist and wheel distributions
- Publish to TestPyPI (if enabled)
- Publish to PyPI
- Create git tag with `sdk/python/v` prefix
- Create GitHub release with changelog

### 5. Post-Release Verification

After the release completes:

1. Verify installation from PyPI:
   ```bash
   pip install ftl-sdk==0.2.0
   ```

2. Test basic functionality:
   ```python
   from ftl_sdk import create_tools, ToolResponse
   print(ToolResponse.text("Hello from v0.2.0!"))
   ```

3. Check the package page: https://pypi.org/project/ftl-sdk/

## Versioning Guidelines

- **Patch releases** (0.1.x): Bug fixes, documentation updates, dependency updates
- **Minor releases** (0.x.0): New features, backwards-compatible changes
- **Major releases** (x.0.0): Breaking changes (note: while in v0.x, breaking changes can happen in minor releases)

## Troubleshooting

### Build Failures

If the build fails:
- Check the error logs in GitHub Actions
- Ensure pyproject.toml is valid
- Verify all tests pass locally with `tox`

### TestPyPI Issues

If TestPyPI installation fails:
- The package name might already exist (can't reuse versions)
- Dependencies might not be available on TestPyPI
- Try with: `pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ ftl-sdk`

### PyPI Publishing Fails

If PyPI publishing fails:
- Verify trusted publisher is configured correctly
- Check that the version doesn't already exist
- Ensure the GitHub environment is properly configured

### Package Not Available Immediately

PyPI can take a few minutes to update. If the package isn't available:
1. Wait 5-10 minutes
2. Try installing with `--no-cache-dir`:
   ```bash
   pip install --no-cache-dir ftl-sdk==0.2.0
   ```

## Manual Release (Emergency Only)

If automation fails, you can release manually:

```bash
cd sdk/python

# Update version in pyproject.toml
# Update CHANGELOG.md

# Build
python -m build

# Upload to TestPyPI
twine upload --repository testpypi dist/*

# Upload to PyPI
twine upload dist/*

# Create git tag
git tag sdk/python/v0.2.0
git push origin sdk/python/v0.2.0
```

**Note**: Manual releases require PyPI API tokens, which should be avoided in favor of trusted publishing.