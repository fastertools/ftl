from ftl_sdk import create_tools, ToolResponse

# Create the handler using FTL SDK
IncomingHandler = create_tools({
    "hello": {
        "description": "Say hello from Python",
        "inputSchema": {
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "Name to greet"}
            },
            "required": ["name"]
        },
        "handler": lambda input: ToolResponse.text(
            f"Hello, {input['name']}! This is FTL Python SDK running on WASM."
        )
    },
    
    "reverse": {
        "description": "Reverse the input text",
        "inputSchema": {
            "type": "object",
            "properties": {
                "text": {"type": "string", "description": "Text to reverse"}
            },
            "required": ["text"]
        },
        "handler": lambda input: ToolResponse.text(input["text"][::-1])
    },
    
    "wordCount": {
        "name": "word_count",  # Explicit name override
        "description": "Count words in text",
        "inputSchema": {
            "type": "object",
            "properties": {
                "text": {"type": "string", "description": "Text to analyze"}
            },
            "required": ["text"]
        },
        "handler": lambda input: ToolResponse.with_structured(
            f"The text contains {len(input['text'].split())} words.",
            {"word_count": len(input["text"].split())}
        )
    }
})