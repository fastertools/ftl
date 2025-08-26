package ui_test

import (
	"os"
	"os/exec"
	"path/filepath"
	"testing"
)

// TestGeneratedCodeExists verifies that generated files exist
func TestGeneratedCodeExists(t *testing.T) {
	// Check current generated files exist
	currentFiles := []string{
		"../internal/ui/strings_gen.go",
		"../internal/ui/styles_gen.go",
	}
	
	for _, file := range currentFiles {
		if _, err := os.Stat(file); os.IsNotExist(err) {
			t.Errorf("Generated file does not exist: %s", file)
		}
	}
}

// TestCUEFilesValid verifies that CUE files are valid
func TestCUEFilesValid(t *testing.T) {
	cueFiles := []string{
		"constants.cue",
		"strings.cue",
		"styles.cue",
	}
	
	for _, file := range cueFiles {
		cmd := exec.Command("cue", "eval", file)
		cmd.Dir = "."
		if output, err := cmd.CombinedOutput(); err != nil {
			t.Errorf("CUE file %s is not valid: %v\nOutput: %s", file, err, output)
		}
	}
}

// TestImportPaths verifies that the project compiles with current imports
func TestImportPaths(t *testing.T) {
	// Try to build the project
	cmd := exec.Command("go", "build", "./...")
	cmd.Dir = filepath.Join("..", "..")
	if output, err := cmd.CombinedOutput(); err != nil {
		t.Errorf("Project does not compile: %v\nOutput: %s", err, output)
	}
}

// TestGenerationConsistency will be used after migration to ensure consistency
func TestGenerationConsistency(t *testing.T) {
	t.Skip("Will be enabled after migration to compare old vs new generation")
	
	// This test will:
	// 1. Save checksums of current generated files
	// 2. Run new generation
	// 3. Compare semantic content (not byte-for-byte)
	// 4. Ensure no functionality is lost
}