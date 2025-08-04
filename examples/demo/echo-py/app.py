from ftl_sdk import FTL

# Create FTL application instance
ftl = FTL()

@ftl.tool(name="echo_py")
def echo_python(message: str) -> str:
    """An MCP tool written in Python that processes messages."""
    return f"Processed: {message}"

@ftl.tool
def add_numbers(a: int, b: int) -> int:
    """Add two numbers and return the result as an integer."""
    return a + b

@ftl.tool 
def get_user_info(name: str, age: int) -> dict:
    """Return user information as a dictionary."""
    return {
        "name": name,
        "age": age,
        "status": "active",
        "metadata": {
            "created": "2025-08-03",
            "type": "user"
        }
    }

@ftl.tool
def is_even(number: int) -> bool:
    """Check if a number is even."""
    return number % 2 == 0

@ftl.tool
def calculate_percentage(value: float, total: float) -> float:
    """Calculate percentage."""
    return (value / total) * 100.0

# Create the Spin handler
IncomingHandler = ftl.create_handler()