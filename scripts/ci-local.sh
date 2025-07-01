#!/bin/bash
set -e

echo "🔍 Running CI checks locally..."
echo ""

echo "📝 Checking formatting..."
cargo fmt-check
echo "✅ Formatting check passed"
echo ""

echo "🔧 Running clippy..."
cargo lint
echo "✅ Clippy passed"
echo ""

echo "🧪 Running tests..."
cargo test-all
echo "✅ Tests passed"
echo ""

echo "🎉 All CI checks passed!"