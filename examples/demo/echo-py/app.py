from ftl_sdk import FTL

# Create FTL application instance
ftl = FTL()

@ftl.tool(name="echo_py")
def echo_python(message: str) -> str:
    """An MCP tool written in Python that processes messages."""
    return f"Processed: {message}"

# Create the Spin handler
IncomingHandler = ftl.create_handler()