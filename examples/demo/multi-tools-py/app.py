from ftl_sdk import create_tools, ToolResponse
import json
from datetime import datetime
import math

def add_numbers(args):
    """Add two numbers together"""
    a = args.get("a", 0)
    b = args.get("b", 0)
    result = a + b
    return ToolResponse.text(f"{a} + {b} = {result}")

def multiply_numbers(args):
    """Multiply two numbers"""
    a = args.get("a", 0)
    b = args.get("b", 0)
    result = a * b
    return ToolResponse.text(f"{a} × {b} = {result}")

def get_current_time(args):
    """Get the current time in a specific format"""
    format_str = args.get("format", "%Y-%m-%d %H:%M:%S")
    try:
        current_time = datetime.now().strftime(format_str)
        return ToolResponse.text(f"Current time: {current_time}")
    except Exception as e:
        return ToolResponse.error(f"Invalid format string: {str(e)}")

def calculate_distance(args):
    """Calculate distance between two points"""
    x1 = args.get("x1", 0)
    y1 = args.get("y1", 0)
    x2 = args.get("x2", 0)
    y2 = args.get("y2", 0)
    
    distance = math.sqrt((x2 - x1)**2 + (y2 - y1)**2)
    return ToolResponse.text(f"Distance between ({x1}, {y1}) and ({x2}, {y2}) is {distance:.2f}")

def json_formatter(args):
    """Format JSON data with pretty printing"""
    data = args.get("data")
    indent = args.get("indent", 2)
    
    if not data:
        return ToolResponse.error("No data provided")
    
    try:
        if isinstance(data, str):
            data = json.loads(data)
        
        formatted = json.dumps(data, indent=indent, sort_keys=True)
        return ToolResponse.text(formatted)
    except Exception as e:
        return ToolResponse.error(f"Failed to format JSON: {str(e)}")

# Define MCP tools - demonstrating multiple tools in one component
IncomingHandler = create_tools({
    "add_py": {
        "description": "Add two numbers",
        "inputSchema": {
            "type": "object",
            "properties": {
                "a": {"type": "number", "description": "First number"},
                "b": {"type": "number", "description": "Second number"}
            },
            "required": ["a", "b"]
        },
        "handler": add_numbers
    },
    "multiply_py": {
        "description": "Multiply two numbers",
        "inputSchema": {
            "type": "object",
            "properties": {
                "a": {"type": "number", "description": "First number"},
                "b": {"type": "number", "description": "Second number"}
            },
            "required": ["a", "b"]
        },
        "handler": multiply_numbers
    },
    "current_time_py": {
        "description": "Get the current time with optional format",
        "inputSchema": {
            "type": "object",
            "properties": {
                "format": {
                    "type": "string", 
                    "description": "Time format string (e.g., '%Y-%m-%d %H:%M:%S')",
                    "default": "%Y-%m-%d %H:%M:%S"
                }
            }
        },
        "handler": get_current_time
    },
    "distance_py": {
        "description": "Calculate Euclidean distance between two points",
        "inputSchema": {
            "type": "object",
            "properties": {
                "x1": {"type": "number", "description": "X coordinate of first point"},
                "y1": {"type": "number", "description": "Y coordinate of first point"},
                "x2": {"type": "number", "description": "X coordinate of second point"},
                "y2": {"type": "number", "description": "Y coordinate of second point"}
            },
            "required": ["x1", "y1", "x2", "y2"]
        },
        "handler": calculate_distance
    },
    "json_formatter_py": {
        "description": "Format JSON data with pretty printing",
        "inputSchema": {
            "type": "object",
            "properties": {
                "data": {
                    "description": "JSON data to format (string or object)"
                },
                "indent": {
                    "type": "number",
                    "description": "Number of spaces for indentation",
                    "default": 2
                }
            },
            "required": ["data"]
        },
        "handler": json_formatter
    }
})