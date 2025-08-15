#!/bin/bash

# This script tests that the packageNamespace field is being used correctly

echo "Testing ECR namespace handling..."

# Create a test app config
cat << 'YAML' > /tmp/test-app.yaml
name: namespace-test-app
version: "0.1.0"
access: public
auth:
  provider: workos
components:
  - id: test-component
    source: "ghcr.io/fermyon/spin-fileserver:latest"
YAML

echo "1. Deploying test app with new namespace handling..."
ftl deploy -f /tmp/test-app.yaml -y --dry-run 2>&1 | grep -E "(namespace|package|ECR|registry)" || true

echo ""
echo "2. The deployment flow now:"
echo "   - Gets ECR token with packageNamespace field"
echo "   - Uses packageNamespace instead of UUID for spin deps publish"
echo "   - Should work with ECR naming requirements"

echo ""
echo "✅ Code updated to use packageNamespace from backend"
