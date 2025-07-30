from ftl_sdk import create_tools, ToolResponse
from spin_sdk import http
import json

def get_weather(args):
    """Get weather for a location using Open-Meteo API"""
    location = args.get("location")
    if not location:
        return ToolResponse.error("Location is required")
    
    # First, get coordinates for the location
    geocoding_url = f"https://geocoding-api.open-meteo.com/v1/search?name={location}"
    
    try:
        geo_response = http.send(http.Request("GET", geocoding_url, {}, None))
        geo_data = json.loads(geo_response.body)
        
        if not geo_data.get("results"):
            return ToolResponse.error(f"Location '{location}' not found")
        
        # Get the first result
        result = geo_data["results"][0]
        lat = result["latitude"]
        lon = result["longitude"]
        name = result["name"]
        country = result.get("country", "")
        
        # Get weather data
        weather_url = f"https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current_weather=true"
        weather_response = http.send(http.Request("GET", weather_url, {}, None))
        weather_data = json.loads(weather_response.body)
        
        current = weather_data["current_weather"]
        
        weather_info = f"Weather for {name}, {country}:\n"
        weather_info += f"Temperature: {current['temperature']}°C\n"
        weather_info += f"Wind Speed: {current['windspeed']} km/h\n"
        weather_info += f"Wind Direction: {current['winddirection']}°"
        
        return ToolResponse.text(weather_info)
        
    except Exception as e:
        return ToolResponse.error(f"Failed to get weather: {str(e)}")

# Define MCP tools
IncomingHandler = create_tools({
    "get_weather_py": {
        "description": "Get current weather for a location",
        "inputSchema": {
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name or location"
                }
            },
            "required": ["location"]
        },
        "handler": get_weather
    }
})