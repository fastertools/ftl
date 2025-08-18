#!/bin/bash

# Update all import paths in Go files to match new structure
echo "Updating import paths..."

# Update imports in all Go files
find . -name "*.go" -type f | while read -r file; do
    # Skip vendor and old go/ directory
    if [[ "$file" == *"/vendor/"* ]] || [[ "$file" == *"/go/"* ]]; then
        continue
    fi
    
    # Update imports
    sed -i 's|github.com/fastertools/ftl-cli/go/ftl/cmd|github.com/fastertools/ftl-cli/internal/cli|g' "$file"
    sed -i 's|github.com/fastertools/ftl-cli/go/ftl/pkg/scaffold|github.com/fastertools/ftl-cli/internal/scaffold|g' "$file"
    sed -i 's|github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis|github.com/fastertools/ftl-cli/internal/synthesis|g' "$file"
    sed -i 's|github.com/fastertools/ftl-cli/go/shared/api|github.com/fastertools/ftl-cli/internal/api|g' "$file"
    sed -i 's|github.com/fastertools/ftl-cli/go/shared/auth|github.com/fastertools/ftl-cli/internal/auth|g' "$file"
    sed -i 's|github.com/fastertools/ftl-cli/go/shared/ftl|github.com/fastertools/ftl-cli/internal/ftl|g' "$file"
    sed -i 's|github.com/fastertools/ftl-cli/go/shared/types|github.com/fastertools/ftl-cli/pkg/types|g' "$file"
    sed -i 's|github.com/fastertools/ftl-cli/go/shared/spin|github.com/fastertools/ftl-cli/pkg/spin|g' "$file"
    sed -i 's|github.com/fastertools/ftl-cli/go/shared|github.com/fastertools/ftl-cli/internal|g' "$file"
done

echo "Import paths updated!"