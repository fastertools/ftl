package ftl

import (
	"fmt"
	"os"
	"os/exec"
	"strings"
)

// Commander handles FTL command execution
type Commander struct {
	executable string
}

// NewCommander creates a new FTL commander with default "ftl" executable
func NewCommander() *Commander {
	return &Commander{
		executable: "ftl",
	}
}

// NewCommanderWithPath creates a new FTL commander with custom executable path
func NewCommanderWithPath(ftlPath string) *Commander {
	return &Commander{
		executable: ftlPath,
	}
}

// SetFTLPath updates the FTL executable path
func (c *Commander) SetFTLPath(path string) {
	c.executable = path
}

// GetExecutable returns the FTL executable path
func (c *Commander) GetExecutable() string {
	return c.executable
}

// BuildCommand constructs an ftl build command
func (c *Commander) BuildCommand(projectPath string, clean bool) *exec.Cmd {
	args := []string{"build"}
	if clean {
		args = append(args, "--clean")
	}
	// Use --project-path flag instead of changing directory
	if projectPath != "" && projectPath != "." {
		args = append(args, "--project-path", projectPath)
	}
	
	fmt.Fprintf(os.Stderr, "DEBUG: Executing command: %s %s\n", 
		c.executable, strings.Join(args, " "))
	
	cmd := exec.Command(c.executable, args...)
	return cmd
}



// ExecuteBuild runs the build command and returns output
func (c *Commander) ExecuteBuild(projectPath string, clean bool) (string, error) {
	cmd := c.BuildCommand(projectPath, clean)
	
	output, err := cmd.CombinedOutput()
	outputStr := string(output)
	
	fmt.Fprintf(os.Stderr, "DEBUG: Command output: %s\n", outputStr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Command error: %v\n", err)
	}
	
	return outputStr, err
}

// ComponentListCommand constructs an ftl component list command
func (c *Commander) ComponentListCommand(projectPath string) *exec.Cmd {
	args := []string{"component", "list"}
	// Use --project-path flag instead of changing directory
	if projectPath != "" && projectPath != "." {
		args = append(args, "--project-path", projectPath)
	}
	
	fmt.Fprintf(os.Stderr, "DEBUG: Executing command: %s %s\n", 
		c.executable, strings.Join(args, " "))
	
	cmd := exec.Command(c.executable, args...)
	return cmd
}

// ExecuteComponentList runs the component list command and returns output
func (c *Commander) ExecuteComponentList(projectPath string) (string, error) {
	cmd := c.ComponentListCommand(projectPath)
	
	output, err := cmd.CombinedOutput()
	outputStr := string(output)
	
	fmt.Fprintf(os.Stderr, "DEBUG: Component list output: %s\n", outputStr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Component list error: %v\n", err)
	}
	
	return outputStr, err
}

// ValidateProjectPath checks if the project directory exists
func ValidateProjectPath(projectPath string) error {
	if _, err := os.Stat(projectPath); os.IsNotExist(err) {
		return fmt.Errorf("project directory '%s' does not exist", projectPath)
	}
	return nil
}