from spin_sdk import http
from spin_sdk.http import Request, Response
import json


class IncomingHandler(http.IncomingHandler):
    def handle_request(self, request: Request) -> Response:
        path = request.uri
        method = request.method
        
        # Handle GET / - return tool metadata
        if method == "GET" and path == "/":
            metadata = [
                {
                    "name": "hello",
                    "description": "Say hello from Python",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string", "description": "Name to greet"}
                        },
                        "required": ["name"]
                    }
                }
            ]
            
            return Response(
                200,
                {"content-type": "application/json"},
                bytes(json.dumps(metadata), "utf-8")
            )
        
        # Handle POST /hello - execute the tool
        elif method == "POST" and path == "/hello":
            try:
                body = request.body.decode("utf-8") if request.body else "{}"
                input_data = json.loads(body)
                name = input_data.get("name", "World")
                
                response = {
                    "content": [{
                        "type": "text",
                        "text": f"Hello, {name}! This is FTL Python running on WASM."
                    }]
                }
                
                return Response(
                    200,
                    {"content-type": "application/json"},
                    bytes(json.dumps(response), "utf-8")
                )
            except Exception as e:
                error_response = {
                    "content": [{
                        "type": "text",
                        "text": f"Error: {str(e)}"
                    }],
                    "isError": True
                }
                return Response(
                    400,
                    {"content-type": "application/json"},
                    bytes(json.dumps(error_response), "utf-8")
                )
        
        # Default response
        return Response(
            404,
            {"content-type": "text/plain"},
            b"Not found"
        )