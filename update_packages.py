#!/usr/bin/env python3
"""
Update package declarations in Go files.
This only updates package declarations, not imports.
"""

import os
import re

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

def update_package(filepath, dry_run=True):
    """Update package declaration in a Go file."""
    
    # Determine expected package based on path
    expected_pkg = None
    for dir_pattern, pkg_name in PACKAGE_MAPPINGS.items():
        if dir_pattern in str(filepath):
            expected_pkg = pkg_name
            break
    
    if not expected_pkg:
        return False
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Find and replace package declaration
    pkg_pattern = r'^package\s+\w+'
    pkg_match = re.search(pkg_pattern, content, re.MULTILINE)
    
    if pkg_match:
        current_declaration = pkg_match.group(0)
        new_declaration = f'package {expected_pkg}'
        
        if current_declaration != new_declaration:
            if dry_run:
                print(f"Would update {filepath}: '{current_declaration}' -> '{new_declaration}'")
                return True
            else:
                new_content = re.sub(pkg_pattern, new_declaration, content, count=1, flags=re.MULTILINE)
                with open(filepath, 'w') as f:
                    f.write(new_content)
                print(f"Updated {filepath}: '{current_declaration}' -> '{new_declaration}'")
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
                    
                    if update_package(filepath, dry_run):
                        updated_files += 1
    
    print("\n" + "=" * 60)
    print(f"{'Would update' if dry_run else 'Updated'} {updated_files} of {total_files} files")

if __name__ == "__main__":
    main()