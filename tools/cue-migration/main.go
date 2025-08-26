package main

import (
	"bytes"
	"fmt"
	"go/ast"
	"go/format"
	"go/parser"
	"go/token"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"strings"

	"golang.org/x/tools/go/ast/astutil"
)

// ImportMapping defines old to new import path mappings
var ImportMapping = map[string]string{
	"github.com/fastertools/ftl/internal/ui": "github.com/fastertools/ftl/ui/internal/gen",
}

// updateFileImports updates imports in a single Go file using AST manipulation
func updateFileImports(filename string, dryRun bool) error {
	fset := token.NewFileSet()
	src, err := ioutil.ReadFile(filename)
	if err != nil {
		return fmt.Errorf("reading file %s: %w", filename, err)
	}

	// Parse the file
	file, err := parser.ParseFile(fset, filename, src, parser.ParseComments)
	if err != nil {
		return fmt.Errorf("parsing file %s: %w", filename, err)
	}

	// Track if any changes were made
	modified := false

	// Update imports using AST manipulation
	for oldPath, newPath := range ImportMapping {
		if astutil.RewriteImportPath(fset, file, oldPath, newPath) {
			modified = true
			fmt.Printf("  Updated import: %s -> %s\n", oldPath, newPath)
		}
	}

	if !modified {
		return nil
	}

	if dryRun {
		fmt.Printf("  [DRY RUN] Would update file: %s\n", filename)
		return nil
	}

	// Format the modified AST
	var buf bytes.Buffer
	if err := format.Node(&buf, fset, file); err != nil {
		return fmt.Errorf("formatting file %s: %w", filename, err)
	}

	// Write back to file
	if err := ioutil.WriteFile(filename, buf.Bytes(), 0644); err != nil {
		return fmt.Errorf("writing file %s: %w", filename, err)
	}

	fmt.Printf("  ✅ Updated file: %s\n", filename)
	return nil
}

// findGoFiles walks the directory tree and finds all Go files
func findGoFiles(root string) ([]string, error) {
	var files []string
	
	err := filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		
		// Skip vendor and .git directories
		if info.IsDir() {
			name := info.Name()
			if name == "vendor" || name == ".git" || strings.HasPrefix(name, ".") {
				return filepath.SkipDir
			}
			return nil
		}
		
		// Only process .go files
		if strings.HasSuffix(path, ".go") && !strings.HasSuffix(path, "_test.go") {
			files = append(files, path)
		}
		
		return nil
	})
	
	return files, err
}

func main() {
	// Parse command line arguments
	dryRun := false
	root := "."
	
	for _, arg := range os.Args[1:] {
		switch arg {
		case "--dry-run":
			dryRun = true
		case "--help":
			fmt.Println("Usage: go run main.go [--dry-run] [path]")
			fmt.Println("  --dry-run  Show what would be changed without modifying files")
			fmt.Println("  path       Root directory to search (default: current directory)")
			os.Exit(0)
		default:
			root = arg
		}
	}
	
	if dryRun {
		fmt.Println("🔍 Running in DRY RUN mode - no files will be modified")
	}
	
	fmt.Printf("📂 Searching for Go files in: %s\n", root)
	
	// Find all Go files
	files, err := findGoFiles(root)
	if err != nil {
		log.Fatalf("Error finding Go files: %v", err)
	}
	
	fmt.Printf("Found %d Go files to process\n\n", len(files))
	
	// Process each file
	updatedCount := 0
	for _, file := range files {
		fmt.Printf("Processing: %s\n", file)
		if err := updateFileImports(file, dryRun); err != nil {
			log.Printf("  ⚠️  Error processing %s: %v", file, err)
			continue
		}
		updatedCount++
	}
	
	fmt.Printf("\n✅ Processed %d/%d files successfully\n", updatedCount, len(files))
	
	if !dryRun {
		fmt.Println("\n🎉 Import migration complete! Run 'go build ./...' to verify compilation.")
	} else {
		fmt.Println("\nRun without --dry-run to apply changes.")
	}
}