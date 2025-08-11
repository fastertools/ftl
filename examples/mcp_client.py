#!/usr/bin/env python3
"""
MCP Client - Python example for calling MCP endpoints with FTL authentication
"""

import subprocess
import json
import requests
import sys
from typing import Any, Dict, Optional


class MCPClient:
    """Client for interacting with MCP endpoints using FTL authentication"""
    
    def __init__(self, endpoint: str = "https://your-app.ftl.tools/mcp", use_m2m: bool = False):
        self.endpoint = endpoint
        self.use_m2m = use_m2m
        self.token = self._get_ftl_token()
        self.request_id = 0
    
    def _get_ftl_token(self) -> str:
        """Get FTL authentication token"""
        try:
            cmd = ['ftl', 'eng', 'auth', 'token']
            if self.use_m2m:
                cmd.append('--m2m')
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                check=True
            )
            return result.stdout.strip()
        except subprocess.CalledProcessError:
            if self.use_m2m:
                raise Exception("M2M authentication failed. Please run 'ftl eng auth token --m2m-setup' first.")
            else:
                raise Exception("Not authenticated. Please run 'ftl login' first.")
    
    def call(self, method: str, params: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Call an MCP method
        
        Args:
            method: The MCP method to call (e.g., "tools/list")
            params: Optional parameters for the method
            
        Returns:
            The JSON-RPC response
        """
        self.request_id += 1
        
        payload = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params or {},
            "id": self.request_id
        }
        
        response = requests.post(
            self.endpoint,
            headers={
                "Authorization": f"Bearer {self.token}",
                "Content-Type": "application/json"
            },
            json=payload
        )
        
        response.raise_for_status()
        return response.json()
    
    def list_tools(self) -> Dict[str, Any]:
        """List available MCP tools"""
        return self.call("tools/list")
    
    def call_tool(self, name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """
        Call a specific MCP tool
        
        Args:
            name: The tool name
            arguments: Tool arguments
            
        Returns:
            The tool response
        """
        return self.call("tools/call", {
            "name": name,
            "arguments": arguments
        })
    
    def list_prompts(self) -> Dict[str, Any]:
        """List available MCP prompts"""
        return self.call("prompts/list")
    
    def get_prompt(self, name: str, arguments: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Get a specific MCP prompt
        
        Args:
            name: The prompt name
            arguments: Optional prompt arguments
            
        Returns:
            The prompt content
        """
        params = {"name": name}
        if arguments:
            params["arguments"] = arguments
        return self.call("prompts/get", params)
    
    def list_resources(self) -> Dict[str, Any]:
        """List available MCP resources"""
        return self.call("resources/list")
    
    def read_resource(self, uri: str) -> Dict[str, Any]:
        """
        Read a specific MCP resource
        
        Args:
            uri: The resource URI
            
        Returns:
            The resource content
        """
        return self.call("resources/read", {"uri": uri})


def main():
    """Example usage of the MCP client"""
    
    # Get endpoint from environment or use default
    import os
    endpoint = os.environ.get("MCP_ENDPOINT", "https://your-app.ftl.tools/mcp")
    use_m2m = os.environ.get("USE_M2M", "false").lower() == "true"
    
    if endpoint == "https://your-app.ftl.tools/mcp":
        print("Note: Using default endpoint. Set MCP_ENDPOINT environment variable to use a different one.")
        print()
    
    if use_m2m:
        print("Using M2M authentication. Set USE_M2M=false to use user authentication.")
        print()
    
    try:
        # Create client
        client = MCPClient(endpoint, use_m2m=use_m2m)
        print(f"Connected to: {endpoint}")
        print("=" * 50)
        print()
        
        # Example 1: List tools
        print("1. Listing available tools:")
        print("-" * 30)
        tools_response = client.list_tools()
        print(json.dumps(tools_response, indent=2))
        print()
        
        # Example 2: List prompts
        print("2. Listing available prompts:")
        print("-" * 30)
        prompts_response = client.list_prompts()
        print(json.dumps(prompts_response, indent=2))
        print()
        
        # Example 3: List resources
        print("3. Listing available resources:")
        print("-" * 30)
        resources_response = client.list_resources()
        print(json.dumps(resources_response, indent=2))
        print()
        
        # Example 4: Call a tool (if any tools are available)
        if tools_response.get("result", {}).get("tools"):
            first_tool = tools_response["result"]["tools"][0]
            print(f"4. Calling tool '{first_tool['name']}':")
            print("-" * 30)
            
            # Prepare arguments based on tool's input schema
            # This is a simple example - you'd need to construct proper arguments
            tool_args = {}
            if first_tool.get("inputSchema", {}).get("properties"):
                # Just use empty/default values for demo
                for prop_name, prop_def in first_tool["inputSchema"]["properties"].items():
                    if prop_def.get("type") == "string":
                        tool_args[prop_name] = "test"
                    elif prop_def.get("type") == "number":
                        tool_args[prop_name] = 0
                    elif prop_def.get("type") == "boolean":
                        tool_args[prop_name] = False
            
            try:
                tool_response = client.call_tool(first_tool["name"], tool_args)
                print(json.dumps(tool_response, indent=2))
            except Exception as e:
                print(f"Error calling tool: {e}")
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()