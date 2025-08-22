package mcpclient

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"os/exec"
	"sync"
	"sync/atomic"
	"time"
)

// Global request ID counter for unique MCP request IDs
var requestIDCounter int64

// Tool represents an MCP tool
type Tool struct {
	Name        string                 `json:"name"`
	Description string                 `json:"description"`
	InputSchema map[string]interface{} `json:"inputSchema"`
}

// Client handles communication with the external FTL MCP server
type Client struct {
	serverPath  string
	serverArgs  []string
	cmd         *exec.Cmd
	stdin       io.WriteCloser
	stdout      io.ReadCloser
	initialized bool
	mu          sync.Mutex // Protects all MCP operations
}

// NewClient creates a new MCP client instance
func NewClient(serverPath string) *Client {
	return &Client{serverPath: serverPath}
}

// NewClientWithArgs creates a new MCP client instance with command arguments
func NewClientWithArgs(serverPath string, args []string) *Client {
	return &Client{
		serverPath: serverPath,
		serverArgs: args,
	}
}

// EnsureConnection ensures we have a persistent connection to the MCP server
func (c *Client) EnsureConnection() error {
	c.mu.Lock()
	defer c.mu.Unlock()
	
	log.Printf("[MCP] EnsureConnection called - checking existing connection...")
	if c.cmd != nil && c.cmd.Process != nil {
		// Process is already running
		log.Printf("[MCP] Connection already exists (PID: %d)", c.cmd.Process.Pid)
		return nil
	}

	// Start persistent MCP server process
	if len(c.serverArgs) > 0 {
		log.Printf("[MCP] Starting new MCP server process: %s %v", c.serverPath, c.serverArgs)
		c.cmd = exec.Command(c.serverPath, c.serverArgs...)
	} else {
		log.Printf("[MCP] Starting new MCP server process: %s", c.serverPath)
		c.cmd = exec.Command(c.serverPath)
	}
	
	stdin, err := c.cmd.StdinPipe()
	if err != nil {
		return fmt.Errorf("failed to create stdin pipe: %v", err)
	}
	c.stdin = stdin
	
	stdout, err := c.cmd.StdoutPipe()
	if err != nil {
		return fmt.Errorf("failed to create stdout pipe: %v", err)
	}
	c.stdout = stdout

	if err := c.cmd.Start(); err != nil {
		log.Printf("[MCP] ERROR: Failed to start MCP server: %v", err)
		return fmt.Errorf("failed to start MCP server: %v", err)
	}
	log.Printf("[MCP] MCP server started successfully (PID: %d)", c.cmd.Process.Pid)

	// Initialize the session once
	initRequest := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      1,
		"method":  "initialize",
		"params": map[string]interface{}{
			"protocolVersion": "2024-11-05",
			"capabilities":    map[string]interface{}{},
			"clientInfo": map[string]interface{}{
				"name":    "web-client",
				"version": "1.0.0",
			},
		},
	}

	log.Printf("[MCP] Sending initialize request...")
	if err := c.sendMessage(c.stdin, initRequest); err != nil {
		log.Printf("[MCP] ERROR: Failed to send initialize: %v", err)
		return fmt.Errorf("failed to send initialize: %v", err)
	}

	// Read initialize response
	log.Printf("[MCP] Reading initialize response...")
	if _, err := c.readMessage(c.stdout); err != nil {
		log.Printf("[MCP] ERROR: Failed to read initialize response: %v", err)
		return fmt.Errorf("failed to read initialize response: %v", err)
	}
	log.Printf("[MCP] Initialize response received successfully")

	// Send initialized notification
	initializedRequest := map[string]interface{}{
		"jsonrpc": "2.0",
		"method":  "notifications/initialized",
	}

	log.Printf("[MCP] Sending initialized notification...")
	if err := c.sendMessage(c.stdin, initializedRequest); err != nil {
		log.Printf("[MCP] ERROR: Failed to send initialized: %v", err)
		return fmt.Errorf("failed to send initialized: %v", err)
	}

	c.initialized = true
	log.Printf("[MCP] Connection established and initialized successfully")
	return nil
}

// CallTool calls an MCP tool with the given arguments
func (c *Client) CallTool(toolName string, args map[string]interface{}) (string, error) {
	// Protect entire tool call operation with mutex
	c.mu.Lock()
	defer c.mu.Unlock()
	
	log.Printf("[MCP] CallTool started: %s with args: %+v", toolName, args)
	
	// Ensure we have a persistent connection (already under mutex)
	if err := c.ensureConnectionUnsafe(); err != nil {
		log.Printf("[MCP] ERROR: Failed to establish connection for %s: %v", toolName, err)
		return "", fmt.Errorf("failed to establish MCP connection: %v", err)
	}

	// Generate unique request ID to avoid race conditions
	requestID := atomic.AddInt64(&requestIDCounter, 1)
	log.Printf("[MCP] Generated request ID %d for tool %s", requestID, toolName)

	// Call the tool on the persistent connection
	toolRequest := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      requestID,
		"method":  "tools/call",
		"params": map[string]interface{}{
			"name":      toolName,
			"arguments": args,
		},
	}
	log.Printf("[MCP] Sending tool request: %+v", toolRequest)

	if err := c.sendMessage(c.stdin, toolRequest); err != nil {
		log.Printf("[MCP] ERROR: Failed to send tool request on first attempt: %v", err)
		// Connection might be broken, try reconnecting once
		c.cleanupUnsafe()
		log.Printf("[MCP] Attempting reconnection for tool %s...", toolName)
		if err := c.ensureConnectionUnsafe(); err != nil {
			log.Printf("[MCP] ERROR: Failed to reconnect for %s: %v", toolName, err)
			return "", fmt.Errorf("failed to reconnect MCP: %v", err)
		}
		log.Printf("[MCP] Retrying tool request after reconnection...")
		if err := c.sendMessage(c.stdin, toolRequest); err != nil {
			log.Printf("[MCP] ERROR: Failed to send tool request on retry: %v", err)
			return "", fmt.Errorf("failed to send tool call: %v", err)
		}
		log.Printf("[MCP] Tool request sent successfully on retry")
	} else {
		log.Printf("[MCP] Tool request sent successfully on first attempt")
	}

	// Read tool response with timeout
	log.Printf("[MCP] Reading response for tool %s (request ID %d)...", toolName, requestID)
	responseChannel := make(chan map[string]interface{}, 1)
	errorChannel := make(chan error, 1)
	
	go func() {
		response, err := c.readMessage(c.stdout)
		if err != nil {
			log.Printf("[MCP] ERROR: Failed to read response for %s: %v", toolName, err)
			errorChannel <- err
		} else {
			log.Printf("[MCP] Response received for %s: %+v", toolName, response)
			responseChannel <- response
		}
	}()
	
	var response map[string]interface{}
	select {
	case response = <-responseChannel:
		log.Printf("[MCP] Successfully received response for %s", toolName)
	case err := <-errorChannel:
		log.Printf("[MCP] ERROR: Failed to read response for %s: %v", toolName, err)
		return "", fmt.Errorf("failed to read tool response: %v", err)
	case <-time.After(15 * time.Second):
		log.Printf("[MCP] ERROR: Tool call %s timed out after 15 seconds", toolName)
		return "", fmt.Errorf("tool call timed out after 15 seconds")
	}

	// Extract result
	if result, ok := response["result"].(map[string]interface{}); ok {
		log.Printf("[MCP] Extracting result from response for %s", toolName)
		if content, ok := result["content"].([]interface{}); ok && len(content) > 0 {
			if textContent, ok := content[0].(map[string]interface{}); ok {
				if text, ok := textContent["text"].(string); ok {
					log.Printf("[MCP] Successfully extracted text result for %s (length: %d)", toolName, len(text))
					return text, nil
				}
			}
		}
	}

	// Check for errors
	if errorObj, ok := response["error"].(map[string]interface{}); ok {
		if message, ok := errorObj["message"].(string); ok {
			log.Printf("[MCP] ERROR: Tool %s returned error: %s", toolName, message)
			return "", fmt.Errorf("MCP error: %s", message)
		}
	}

	log.Printf("[MCP] WARNING: No valid response from MCP server for tool %s", toolName)
	return "No response from MCP server", nil
}

// Cleanup cleans up the MCP client resources
func (c *Client) Cleanup() {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.cleanupUnsafe()
}

// cleanupUnsafe performs cleanup without acquiring mutex (for internal use)
func (c *Client) cleanupUnsafe() {
	log.Printf("[MCP] Cleaning up MCP client resources...")
	if c.cmd != nil && c.cmd.Process != nil {
		log.Printf("[MCP] Killing MCP server process (PID: %d)", c.cmd.Process.Pid)
		c.cmd.Process.Kill()
		c.cmd.Wait()
	}
	c.cmd = nil
	c.stdin = nil
	c.stdout = nil
	c.initialized = false
	log.Printf("[MCP] Cleanup completed")
}

// ensureConnectionUnsafe ensures connection without acquiring mutex (for internal use)
func (c *Client) ensureConnectionUnsafe() error {
	log.Printf("[MCP] ensureConnectionUnsafe called - checking existing connection...")
	
	if c.cmd != nil && c.cmd.Process != nil {
		// Process is already running
		log.Printf("[MCP] Connection already exists (PID: %d)", c.cmd.Process.Pid)
		return nil
	}

	// Start persistent MCP server process
	if len(c.serverArgs) > 0 {
		log.Printf("[MCP] Starting new MCP server process: %s %v", c.serverPath, c.serverArgs)
		c.cmd = exec.Command(c.serverPath, c.serverArgs...)
	} else {
		log.Printf("[MCP] Starting new MCP server process: %s", c.serverPath)
		c.cmd = exec.Command(c.serverPath)
	}
	
	stdin, err := c.cmd.StdinPipe()
	if err != nil {
		return fmt.Errorf("failed to create stdin pipe: %v", err)
	}
	c.stdin = stdin
	
	stdout, err := c.cmd.StdoutPipe()
	if err != nil {
		return fmt.Errorf("failed to create stdout pipe: %v", err)
	}
	c.stdout = stdout

	if err := c.cmd.Start(); err != nil {
		log.Printf("[MCP] ERROR: Failed to start MCP server: %v", err)
		return fmt.Errorf("failed to start MCP server: %v", err)
	}
	log.Printf("[MCP] MCP server started successfully (PID: %d)", c.cmd.Process.Pid)

	// Initialize the session once
	initRequest := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      1,
		"method":  "initialize",
		"params": map[string]interface{}{
			"protocolVersion": "2024-11-05",
			"capabilities":    map[string]interface{}{},
			"clientInfo": map[string]interface{}{
				"name":    "web-client",
				"version": "1.0.0",
			},
		},
	}

	log.Printf("[MCP] Sending initialize request...")
	if err := c.sendMessage(c.stdin, initRequest); err != nil {
		log.Printf("[MCP] ERROR: Failed to send initialize: %v", err)
		return fmt.Errorf("failed to send initialize: %v", err)
	}

	// Read initialize response
	log.Printf("[MCP] Reading initialize response...")
	if _, err := c.readMessage(c.stdout); err != nil {
		log.Printf("[MCP] ERROR: Failed to read initialize response: %v", err)
		return fmt.Errorf("failed to read initialize response: %v", err)
	}
	log.Printf("[MCP] Initialize response received successfully")

	// Send initialized notification
	initializedRequest := map[string]interface{}{
		"jsonrpc": "2.0",
		"method":  "notifications/initialized",
	}

	log.Printf("[MCP] Sending initialized notification...")
	if err := c.sendMessage(c.stdin, initializedRequest); err != nil {
		log.Printf("[MCP] ERROR: Failed to send initialized: %v", err)
		return fmt.Errorf("failed to send initialized: %v", err)
	}

	c.initialized = true
	log.Printf("[MCP] Connection established and initialized successfully")
	return nil
}

func (c *Client) sendMessage(stdin io.WriteCloser, message map[string]interface{}) error {
	data, err := json.Marshal(message)
	if err != nil {
		log.Printf("[MCP] ERROR: Failed to marshal message: %v", err)
		return err
	}
	
	log.Printf("[MCP] Sending message (length: %d bytes): %s", len(data), string(data))
	
	if _, err := stdin.Write(data); err != nil {
		log.Printf("[MCP] ERROR: Failed to write message data: %v", err)
		return err
	}
	if _, err := stdin.Write([]byte("\n")); err != nil {
		log.Printf("[MCP] ERROR: Failed to write newline: %v", err)
		return err
	}
	log.Printf("[MCP] Message sent successfully")
	return nil
}

func (c *Client) readMessage(stdout io.ReadCloser) (map[string]interface{}, error) {
	log.Printf("[MCP] Starting to read message from stdout...")
	decoder := json.NewDecoder(stdout)
	var response map[string]interface{}
	if err := decoder.Decode(&response); err != nil {
		log.Printf("[MCP] ERROR: Failed to decode response: %v", err)
		return nil, err
	}
	log.Printf("[MCP] Message read successfully: %+v", response)
	return response, nil
}