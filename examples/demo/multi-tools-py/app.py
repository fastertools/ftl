from ftl_sdk import FTL
import json
from datetime import datetime
import math
from typing import Union, Optional, Any

# Create FTL application instance
ftl = FTL()

@ftl.tool(name="add_py")
def add_numbers(a: float, b: float) -> str:
    """Add two numbers together."""
    result = a + b
    return f"{a} + {b} = {result}"

@ftl.tool(name="multiply_py")
def multiply_numbers(a: float, b: float) -> str:
    """Multiply two numbers."""
    result = a * b
    return f"{a} × {b} = {result}"

@ftl.tool(name="current_time_py")
def get_current_time(format: str = "%Y-%m-%d %H:%M:%S") -> str:
    """Get the current time with optional format string."""
    try:
        current_time = datetime.now().strftime(format)
        return f"Current time: {current_time}"
    except Exception as e:
        raise ValueError(f"Invalid format string: {str(e)}")

@ftl.tool(name="distance_py")
def calculate_distance(x1: float, y1: float, x2: float, y2: float) -> str:
    """Calculate Euclidean distance between two points."""
    distance = math.sqrt((x2 - x1)**2 + (y2 - y1)**2)
    return f"Distance between ({x1}, {y1}) and ({x2}, {y2}) is {distance:.2f}"

@ftl.tool(name="json_formatter_py")
def json_formatter(data: Union[str, dict, list], indent: int = 2) -> str:
    """Format JSON data with pretty printing."""
    if isinstance(data, str):
        try:
            data = json.loads(data)
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON string: {str(e)}")
    
    return json.dumps(data, indent=indent, sort_keys=True)

# Create the Spin handler
IncomingHandler = ftl.create_handler()