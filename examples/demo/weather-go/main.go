package main

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"strings"
	
	ftl "github.com/fastertools/ftl-cli/sdk/go"
)

// secureRandomInt generates a cryptographically secure random integer in range [0, max)
func secureRandomInt(max int) int {
	n, err := rand.Int(rand.Reader, big.NewInt(int64(max)))
	if err != nil {
		// Fallback to 0 if crypto/rand fails (should never happen)
		return 0
	}
	return int(n.Int64())
}

func init() {
	ftl.CreateTools(map[string]ftl.ToolDefinition{
		"check_weather": {
			Description: "Check the weather for a given location",
			InputSchema: map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"location": map[string]interface{}{
						"type":        "string",
						"description": "The location to check weather for",
					},
					"unit": map[string]interface{}{
						"type":        "string",
						"description": "Temperature unit (celsius or fahrenheit)",
						"enum":        []string{"celsius", "fahrenheit"},
						"default":     "celsius",
					},
				},
				"required": []string{"location"},
			},
			Handler: checkWeatherHandler,
		},
		"get_forecast": {
			Description: "Get weather forecast for the next few days",
			InputSchema: map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"location": map[string]interface{}{
						"type":        "string",
						"description": "The location to get forecast for",
					},
					"days": map[string]interface{}{
						"type":        "integer",
						"description": "Number of days to forecast (1-7)",
						"minimum":     1,
						"maximum":     7,
						"default":     3,
					},
				},
				"required": []string{"location"},
			},
			Handler: getForecastHandler,
		},
	})
}

func checkWeatherHandler(input map[string]interface{}) ftl.ToolResponse {
	location, _ := input["location"].(string)
	unit, ok := input["unit"].(string)
	if !ok {
		unit = "celsius"
	}
	
	// Simulate weather data (in a real implementation, this would call a weather API)
	temp := secureRandomInt(30) + 10 // 10-40 degrees
	if unit == "fahrenheit" {
		temp = (temp * 9 / 5) + 32
	}
	
	conditions := []string{"Sunny", "Partly Cloudy", "Cloudy", "Rainy", "Stormy"}
	condition := conditions[secureRandomInt(len(conditions))]
	
	humidity := secureRandomInt(60) + 30 // 30-90%
	windSpeed := secureRandomInt(20) + 5 // 5-25 km/h
	
	unitSymbol := "°C"
	if unit == "fahrenheit" {
		unitSymbol = "°F"
	}
	
	weatherReport := fmt.Sprintf(
		"Current weather in %s:\n"+
		"Temperature: %d%s\n"+
		"Condition: %s\n"+
		"Humidity: %d%%\n"+
		"Wind Speed: %d km/h",
		location, temp, unitSymbol, condition, humidity, windSpeed,
	)
	
	return ftl.WithStructured(weatherReport, map[string]interface{}{
		"location":    location,
		"temperature": temp,
		"unit":        unit,
		"condition":   strings.ToLower(condition),
		"humidity":    humidity,
		"windSpeed":   windSpeed,
	})
}

func getForecastHandler(input map[string]interface{}) ftl.ToolResponse {
	location, _ := input["location"].(string)
	days := 3
	if d, ok := input["days"].(float64); ok { // JSON numbers are float64
		days = int(d)
		if days < 1 {
			days = 1
		} else if days > 7 {
			days = 7
		}
	}
	
	conditions := []string{"Sunny", "Partly Cloudy", "Cloudy", "Rainy", "Stormy"}
	weekdays := []string{"Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"}
	
	var forecast []string
	var forecastData []map[string]interface{}
	
	for i := 0; i < days; i++ {
		condition := conditions[secureRandomInt(len(conditions))]
		high := secureRandomInt(15) + 20 // 20-35°C
		low := high - secureRandomInt(10) - 5 // 5-15 degrees lower
		
		dayForecast := fmt.Sprintf("Day %d (%s): %s, High: %d°C, Low: %d°C",
			i+1, weekdays[i%7], condition, high, low)
		forecast = append(forecast, dayForecast)
		
		forecastData = append(forecastData, map[string]interface{}{
			"day":       i + 1,
			"weekday":   weekdays[i%7],
			"condition": strings.ToLower(condition),
			"high":      high,
			"low":       low,
		})
	}
	
	forecastText := fmt.Sprintf("%d-day forecast for %s:\n%s",
		days, location, strings.Join(forecast, "\n"))
	
	return ftl.WithStructured(forecastText, map[string]interface{}{
		"location": location,
		"days":     days,
		"forecast": forecastData,
	})
}

func main() {
	// Required by TinyGo but not used
}