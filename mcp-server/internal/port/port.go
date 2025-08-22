package port

import (
	"fmt"
	"net"
	"os"
)

// Manager handles port allocation for FTL processes
type Manager struct {
	startPort int
}

// NewManager creates a new port manager
func NewManager(startPort int) *Manager {
	return &Manager{
		startPort: startPort,
	}
}

// FindAvailable finds an available port starting from the configured start port
// Always checks the current port first, then increments
func (m *Manager) FindAvailable() (int, error) {
	return FindAvailablePort(m.startPort)
}

// FindAvailablePort finds an available port starting from the given port
func FindAvailablePort(startPort int) (int, error) {
	for port := startPort; port < startPort+100; port++ {
		if IsAvailable(port) {
			return port, nil
		}
	}
	return 0, fmt.Errorf("no available ports found in range %d-%d", startPort, startPort+99)
}

// IsAvailable checks if a specific port is available
func IsAvailable(port int) bool {
	// Test the exact address that FTL will try to bind to
	addr := fmt.Sprintf("127.0.0.1:%d", port)
	listener, err := net.Listen("tcp", addr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Port %d not available: %v\n", port, err)
		return false
	}
	listener.Close()
	fmt.Fprintf(os.Stderr, "Port %d is available\n", port)
	return true
}