package ftl

import (
	"fmt"
	"os"
	"os/exec"
	"strings"
	"syscall"
)

// Commander handles FTL command execution
type Commander struct {
	executable string
}

// NewCommander creates a new FTL commander
func NewCommander() *Commander {
	return &Commander{
		executable: "ftl",
	}
}

// BuildCommand constructs an ftl build command
func (c *Commander) BuildCommand(projectPath string, clean bool) *exec.Cmd {
	args := []string{"build"}
	if clean {
		args = append(args, "--clean")
	}
	
	fmt.Fprintf(os.Stderr, "DEBUG: Executing command: %s %s in directory: %s\n", 
		c.executable, strings.Join(args, " "), projectPath)
	
	cmd := exec.Command(c.executable, args...)
	cmd.Dir = projectPath
	return cmd
}

// UpCommand constructs an ftl up command with optional watch mode
func (c *Commander) UpCommand(projectPath string, listen string, build bool, watch bool) *exec.Cmd {
	args := []string{"up"}
	
	// Add watch flag if requested
	if watch {
		args = append(args, "--watch")
	}
	
	// Add build flag if requested
	if build {
		args = append(args, "--build")
	}
	
	// Add listen address
	args = append(args, "--listen", listen)
	
	cmd := exec.Command(c.executable, args...)
	cmd.Dir = projectPath
	
	// For regular "up" mode (not watch), detach the process so it survives console exit
	if !watch {
		cmd.SysProcAttr = &syscall.SysProcAttr{
			Setpgid: true,  // Create new process group
		}
	}
	
	return cmd
}

// UpWatchCommand constructs an ftl up --watch command (legacy compatibility)
func (c *Commander) UpWatchCommand(projectPath string, port int, build bool) *exec.Cmd {
	listenAddr := fmt.Sprintf("localhost:%d", port)
	return c.UpCommand(projectPath, listenAddr, build, true)
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
	fmt.Fprintf(os.Stderr, "DEBUG: Executing command: %s component list in directory: %s\n", 
		c.executable, projectPath)
	
	cmd := exec.Command(c.executable, "component", "list")
	cmd.Dir = projectPath
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