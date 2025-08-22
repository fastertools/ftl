package network

import (
	"fmt"
	"net"
	"net/http"
)

// IsPortAvailable checks if a specific port is available
func IsPortAvailable(port int) bool {
	addr := fmt.Sprintf("127.0.0.1:%d", port)
	listener, err := net.Listen("tcp", addr)
	if err != nil {
		return false
	}
	listener.Close()
	return true
}

// StartServerOnAvailablePort tries to start the server on available ports
func StartServerOnAvailablePort(startPort int, handler http.Handler) error {
	for port := startPort; port < startPort+10; port++ {
		addr := fmt.Sprintf(":%d", port)
		fmt.Printf("🔍 Attempting to bind to port %d...\n", port)
		
		listener, err := net.Listen("tcp", addr)
		if err != nil {
			fmt.Printf("❌ Port %d busy: %v\n", port, err)
			continue
		}
		
		fmt.Printf("✅ Successfully bound to port %d\n", port)
		fmt.Printf("🚀 FTL Web Interface starting on http://localhost:%d\n", port)
		fmt.Println("📋 Available endpoints:")
		fmt.Printf("   GET  / - Web interface\n")
		fmt.Printf("   POST /mcp - All MCP operations (ftl up, watch start/stop)\n")
		fmt.Printf("   GET  /mcp?since=N - Get logs from watch process\n")
		fmt.Println()
		fmt.Println("💡 MCP Server: ./mcp-server/console-server")
		fmt.Println("   Build it first: cd mcp-server && go build -o console-server")
		
		// Serve using the existing listener
		return http.Serve(listener, handler)
	}
	
	return fmt.Errorf("no available ports found in range %d-%d", startPort, startPort+9)
}