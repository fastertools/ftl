from ftl_sdk import FTL
from spin_sdk import http
import json

# Create FTL application instance
ftl = FTL()

@ftl.tool(name="get_weather_py")
def get_weather(location: str) -> str:
    """Get current weather for a location using Open-Meteo API."""
    # First, get coordinates for the location
    geocoding_url = f"https://geocoding-api.open-meteo.com/v1/search?name={location}"
    
    try:
        geo_response = http.send(http.Request("GET", geocoding_url, {}, None))
        geo_data = json.loads(geo_response.body)
        
        if not geo_data.get("results"):
            raise ValueError(f"Location '{location}' not found")
        
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
        
        return weather_info
        
    except Exception as e:
        raise ValueError(f"Failed to get weather: {str(e)}")

# Create the Spin handler
IncomingHandler = ftl.create_handler()