#!/bin/bash
set -e

echo "🔍 Running CI checks locally..."
echo ""

echo "📝 Checking formatting..."
just fmt-check
echo "✅ Formatting check passed"
echo ""

echo "🔧 Running clippy..."
just lint
echo "✅ Clippy passed"
echo ""

echo "🧪 Running tests..."
just test-all
echo "✅ Tests passed"
echo ""

echo "🎉 All CI checks passed!"