#!/usr/bin/env python3
"""
Update import statements in Go files.
"""

import os
import re

# Import mappings - old path to new path
IMPORT_MAPPINGS = [
    ('github.com/fastertools/ftl-cli/go/ftl/cmd', 'github.com/fastertools/ftl-cli/internal/cli'),
    ('github.com/fastertools/ftl-cli/go/ftl/pkg/scaffold', 'github.com/fastertools/ftl-cli/internal/scaffold'),
    ('github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis', 'github.com/fastertools/ftl-cli/internal/synthesis'),
    ('github.com/fastertools/ftl-cli/go/shared/api', 'github.com/fastertools/ftl-cli/internal/api'),
    ('github.com/fastertools/ftl-cli/go/shared/auth', 'github.com/fastertools/ftl-cli/internal/auth'),
    ('github.com/fastertools/ftl-cli/go/shared/ftl', 'github.com/fastertools/ftl-cli/internal/ftl'),
    ('github.com/fastertools/ftl-cli/go/shared/types', 'github.com/fastertools/ftl-cli/pkg/types'),
    ('github.com/fastertools/ftl-cli/go/shared/spin', 'github.com/fastertools/ftl-cli/pkg/spin'),
]

def update_imports(filepath, dry_run=True):
    """Update import statements in a Go file."""
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    changes_made = []
    new_content = content
    
    for old_import, new_import in IMPORT_MAPPINGS:
        if old_import in content:
            # Handle both quoted and unquoted imports
            pattern1 = f'"{old_import}"'
            replacement1 = f'"{new_import}"'
            
            if pattern1 in new_content:
                new_content = new_content.replace(pattern1, replacement1)
                changes_made.append(f"  {old_import} -> {new_import}")
    
    if changes_made:
        if dry_run:
            print(f"{filepath}:")
            for change in changes_made:
                print(change)
            return True
        else:
            with open(filepath, 'w') as f:
                f.write(new_content)
            print(f"Updated {filepath}:")
            for change in changes_made:
                print(change)
            return True
    
    return False

def main():
    """Main function."""
    import sys
    
    dry_run = '--dry-run' in sys.argv or len(sys.argv) == 1
    
    if dry_run:
        print("DRY RUN MODE - No files will be modified")
        print("Run with --execute to actually update files")
        print("=" * 60)
    
    dirs_to_check = ['internal/', 'pkg/', 'cmd/']
    
    total_files = 0
    updated_files = 0
    
    for dir_name in dirs_to_check:
        if not os.path.exists(dir_name):
            continue
            
        for root, _, files in os.walk(dir_name):
            for file in files:
                if file.endswith('.go'):
                    filepath = os.path.join(root, file)
                    total_files += 1
                    
                    if update_imports(filepath, dry_run):
                        updated_files += 1
    
    print("\n" + "=" * 60)
    print(f"{'Would update' if dry_run else 'Updated'} {updated_files} of {total_files} files")

if __name__ == "__main__":
    main()