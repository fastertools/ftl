package main

import (
	"encoding/json"
	"fmt"
	"math"
	"strings"
	
	ftl "github.com/fastertools/ftl-cli/sdk/go"
)

func init() {
	ftl.CreateTools(map[string]ftl.ToolDefinition{
		"string_reverse": {
			Description: "Reverse a string",
			InputSchema: map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"text": map[string]interface{}{
						"type":        "string",
						"description": "The string to reverse",
					},
				},
				"required": []string{"text"},
			},
			Handler: func(input map[string]interface{}) ftl.ToolResponse {
				text, _ := input["text"].(string)
				runes := []rune(text)
				for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
					runes[i], runes[j] = runes[j], runes[i]
				}
				reversed := string(runes)
				
				return ftl.WithStructured(
					fmt.Sprintf("Reversed: %s", reversed),
					map[string]interface{}{
						"original": text,
						"reversed": reversed,
						"length":   len(text),
					},
				)
			},
		},
		"word_count": {
			Description: "Count words in a text",
			InputSchema: map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"text": map[string]interface{}{
						"type":        "string",
						"description": "The text to analyze",
					},
					"caseSensitive": map[string]interface{}{
						"type":        "boolean",
						"description": "Whether to consider case when counting",
						"default":     false,
					},
				},
				"required": []string{"text"},
			},
			Handler: wordCountHandler,
		},
		"json_parse": {
			Description: "Parse and validate JSON",
			InputSchema: map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"json": map[string]interface{}{
						"type":        "string",
						"description": "JSON string to parse",
					},
					"indent": map[string]interface{}{
						"type":        "boolean",
						"description": "Pretty print the output",
						"default":     true,
					},
				},
				"required": []string{"json"},
			},
			Handler: jsonParseHandler,
		},
		"calculate": {
			Description: "Perform basic calculations",
			InputSchema: map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"operation": map[string]interface{}{
						"type":        "string",
						"description": "The operation to perform",
						"enum":        []string{"add", "subtract", "multiply", "divide", "power", "sqrt"},
					},
					"a": map[string]interface{}{
						"type":        "number",
						"description": "First operand",
					},
					"b": map[string]interface{}{
						"type":        "number",
						"description": "Second operand (not needed for sqrt)",
					},
				},
				"required": []string{"operation", "a"},
			},
			Handler: calculateHandler,
		},
	})
}

func wordCountHandler(input map[string]interface{}) ftl.ToolResponse {
	text, _ := input["text"].(string)
	caseSensitive := false
	if cs, ok := input["caseSensitive"].(bool); ok {
		caseSensitive = cs
	}
	
	// Count words
	words := strings.Fields(text)
	wordCount := len(words)
	
	// Count unique words
	wordMap := make(map[string]int)
	for _, word := range words {
		if !caseSensitive {
			word = strings.ToLower(word)
		}
		// Remove common punctuation
		word = strings.Trim(word, ".,!?;:\"'")
		if word != "" {
			wordMap[word]++
		}
	}
	
	// Find most common words
	type wordFreq struct {
		word  string
		count int
	}
	var frequencies []wordFreq
	for word, count := range wordMap {
		frequencies = append(frequencies, wordFreq{word, count})
	}
	
	// Simple sort to find top words (in real implementation, would use proper sorting)
	var topWords []string
	maxCount := 0
	for _, wf := range frequencies {
		if wf.count > maxCount {
			maxCount = wf.count
			topWords = []string{wf.word}
		} else if wf.count == maxCount {
			topWords = append(topWords, wf.word)
		}
	}
	
	summary := fmt.Sprintf(
		"Word count: %d\n"+
		"Unique words: %d\n"+
		"Characters: %d\n"+
		"Most frequent: %s (%d times)",
		wordCount, len(wordMap), len(text),
		strings.Join(topWords, ", "), maxCount,
	)
	
	return ftl.WithStructured(summary, map[string]interface{}{
		"wordCount":     wordCount,
		"uniqueWords":   len(wordMap),
		"characterCount": len(text),
		"mostFrequent":  topWords,
		"frequency":     maxCount,
		"caseSensitive": caseSensitive,
	})
}

func jsonParseHandler(input map[string]interface{}) ftl.ToolResponse {
	jsonStr, _ := input["json"].(string)
	indent := true
	if ind, ok := input["indent"].(bool); ok {
		indent = ind
	}
	
	// Parse JSON
	var data interface{}
	if err := json.Unmarshal([]byte(jsonStr), &data); err != nil {
		return ftl.Errorf("Invalid JSON: %v", err)
	}
	
	// Analyze structure
	var analysis strings.Builder
	analysis.WriteString("JSON parsed successfully!\n\n")
	
	switch v := data.(type) {
	case map[string]interface{}:
		analysis.WriteString(fmt.Sprintf("Type: Object with %d keys\n", len(v)))
		analysis.WriteString("Keys: ")
		keys := make([]string, 0, len(v))
		for k := range v {
			keys = append(keys, k)
		}
		analysis.WriteString(strings.Join(keys, ", "))
	case []interface{}:
		analysis.WriteString(fmt.Sprintf("Type: Array with %d elements", len(v)))
	case string:
		analysis.WriteString(fmt.Sprintf("Type: String (length: %d)", len(v)))
	case float64:
		analysis.WriteString(fmt.Sprintf("Type: Number (value: %v)", v))
	case bool:
		analysis.WriteString(fmt.Sprintf("Type: Boolean (value: %v)", v))
	case nil:
		analysis.WriteString("Type: Null")
	}
	
	// Format output
	var formatted []byte
	var err error
	if indent {
		formatted, err = json.MarshalIndent(data, "", "  ")
	} else {
		formatted, err = json.Marshal(data)
	}
	
	if err != nil {
		return ftl.Errorf("Failed to format JSON: %v", err)
	}
	
	result := fmt.Sprintf("%s\n\nFormatted JSON:\n%s", analysis.String(), string(formatted))
	
	return ftl.WithStructured(result, map[string]interface{}{
		"valid":     true,
		"formatted": string(formatted),
		"analysis":  analysis.String(),
	})
}

func calculateHandler(input map[string]interface{}) ftl.ToolResponse {
	operation, _ := input["operation"].(string)
	a, _ := input["a"].(float64)
	b, _ := input["b"].(float64)
	
	var result float64
	var explanation string
	
	switch operation {
	case "add":
		result = a + b
		explanation = fmt.Sprintf("%.2f + %.2f = %.2f", a, b, result)
	case "subtract":
		result = a - b
		explanation = fmt.Sprintf("%.2f - %.2f = %.2f", a, b, result)
	case "multiply":
		result = a * b
		explanation = fmt.Sprintf("%.2f × %.2f = %.2f", a, b, result)
	case "divide":
		if b == 0 {
			return ftl.Error("Cannot divide by zero")
		}
		result = a / b
		explanation = fmt.Sprintf("%.2f ÷ %.2f = %.2f", a, b, result)
	case "power":
		result = math.Pow(a, b)
		explanation = fmt.Sprintf("%.2f ^ %.2f = %.2f", a, b, result)
	case "sqrt":
		if a < 0 {
			return ftl.Error("Cannot take square root of negative number")
		}
		result = math.Sqrt(a)
		explanation = fmt.Sprintf("√%.2f = %.2f", a, result)
	default:
		return ftl.Errorf("Unknown operation: %s", operation)
	}
	
	return ftl.WithStructured(explanation, map[string]interface{}{
		"operation": operation,
		"a":         a,
		"b":         b,
		"result":    result,
		"expression": explanation,
	})
}

func main() {
	// Required by TinyGo but not used
}