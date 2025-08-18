#!/usr/bin/env python3
"""
Analyze Go files and show what imports need updating.
This script only reads files and shows what needs to be changed.
"""

import os
import re
from pathlib import Path

# Import mappings
IMPORT_MAPPINGS = {
    'github.com/fastertools/ftl-cli/go/ftl/cmd': 'github.com/fastertools/ftl-cli/internal/cli',
    'github.com/fastertools/ftl-cli/go/ftl/pkg/scaffold': 'github.com/fastertools/ftl-cli/internal/scaffold',
    'github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis': 'github.com/fastertools/ftl-cli/internal/synthesis',
    'github.com/fastertools/ftl-cli/go/shared/api': 'github.com/fastertools/ftl-cli/internal/api',
    'github.com/fastertools/ftl-cli/go/shared/auth': 'github.com/fastertools/ftl-cli/internal/auth',
    'github.com/fastertools/ftl-cli/go/shared/ftl': 'github.com/fastertools/ftl-cli/internal/ftl',
    'github.com/fastertools/ftl-cli/go/shared/types': 'github.com/fastertools/ftl-cli/pkg/types',
    'github.com/fastertools/ftl-cli/go/shared/spin': 'github.com/fastertools/ftl-cli/pkg/spin',
}

# Package name mappings for package declarations
PACKAGE_MAPPINGS = {
    'internal/cli': 'cli',
    'internal/scaffold': 'scaffold', 
    'internal/synthesis': 'synthesis',
    'internal/api': 'api',
    'internal/auth': 'auth',
    'internal/ftl': 'ftl',
    'pkg/types': 'types',
    'pkg/spin': 'spin',
}

def analyze_file(filepath):
    """Analyze a single Go file for needed changes."""
    changes = []
    
    with open(filepath, 'r') as f:
        content = f.read()
        
    # Check package declaration
    for dir_pattern, expected_pkg in PACKAGE_MAPPINGS.items():
        if dir_pattern in str(filepath):
            pkg_match = re.search(r'^package\s+(\w+)', content, re.MULTILINE)
            if pkg_match:
                current_pkg = pkg_match.group(1)
                if current_pkg != expected_pkg:
                    changes.append(f"  Package: '{current_pkg}' -> '{expected_pkg}'")
    
    # Check imports
    for old_import, new_import in IMPORT_MAPPINGS.items():
        if old_import in content:
            changes.append(f"  Import: '{old_import}' -> '{new_import}'")
    
    return changes

def main():
    """Main function to analyze all Go files."""
    dirs_to_check = ['internal/', 'pkg/', 'cmd/']
    
    print("Analyzing Go files for import updates needed...")
    print("=" * 60)
    
    total_files = 0
    files_needing_changes = 0
    
    for dir_name in dirs_to_check:
        if not os.path.exists(dir_name):
            continue
            
        for root, _, files in os.walk(dir_name):
            for file in files:
                if file.endswith('.go'):
                    filepath = os.path.join(root, file)
                    total_files += 1
                    
                    changes = analyze_file(filepath)
                    if changes:
                        files_needing_changes += 1
                        print(f"\n{filepath}:")
                        for change in changes:
                            print(change)
    
    print("\n" + "=" * 60)
    print(f"Summary: {files_needing_changes} of {total_files} files need updates")
    print("\nThis script only analyzes - it does not modify any files.")
    print("Review the changes above before proceeding with updates.")

if __name__ == "__main__":
    main()