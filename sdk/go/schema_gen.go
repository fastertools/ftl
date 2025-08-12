// Package ftl - Schema Generation for V3 Type-Safe Handlers
//
// This file provides automatic JSON schema generation from Go struct tags,
// following standard Go patterns similar to encoding/json.
package ftl

import (
	"fmt"
	"reflect"
	"strconv"
	"strings"
)

// generateSchema creates JSON schema from Go struct tags.
// This is the core function that enables automatic schema generation
// for V3 type-safe handlers.
//
// Supported struct tags:
//   - `json:"field_name,omitempty"` - Controls JSON field names and optional fields
//   - `jsonschema:"required,description=...,minimum=1,maximum=10"` - Schema constraints
//
// Example struct:
//
//	type Input struct {
//	    Name string `json:"name" jsonschema:"required,description=User name"`
//	    Age  int    `json:"age,omitempty" jsonschema:"minimum=0,maximum=120"`
//	}
func generateSchema[T any]() map[string]interface{} {
	var zero T
	t := reflect.TypeOf(zero)
	
	// For CRAWL phase, we'll implement basic struct handling
	// RUN phase will add full reflection-based schema generation
	if t.Kind() != reflect.Struct {
		// Non-struct types get basic schema for now
		return generateScalarSchema(t)
	}
	
	return generateStructSchema(t)
}

// generateStructSchema creates schema for struct types
func generateStructSchema(t reflect.Type) map[string]interface{} {
	// Create a stack to track the current path for circular reference detection
	stack := make([]reflect.Type, 0)
	return generateStructSchemaWithStack(t, stack)
}

// generateStructSchemaWithStack creates schema for struct types with proper circular reference detection
func generateStructSchemaWithStack(t reflect.Type, stack []reflect.Type) map[string]interface{} {
	// Check for circular references by scanning the current stack
	for _, stackType := range stack {
		if stackType == t {
			// Return a reference schema to break the cycle
			return map[string]interface{}{
				"type":        "object",
				"description": fmt.Sprintf("Circular reference to %s", t.Name()),
			}
		}
	}
	
	// Add current type to stack
	newStack := append(stack, t)
	
	properties := make(map[string]interface{})
	required := []string{}
	
	for i := 0; i < t.NumField(); i++ {
		field := t.Field(i)
		
		// Skip unexported fields
		if !field.IsExported() {
			continue
		}
		
		// Get JSON field name
		jsonName := getJSONFieldName(field)
		if jsonName == "-" || jsonName == "" {
			continue // Skip excluded fields and fields without json tags
		}
		
		// Generate field schema with full support
		fieldSchema := generateFieldSchemaWithStack(field, newStack)
		
		// Add schema tag constraints
		schemaProps := parseSchemaTag(field.Tag.Get("jsonschema"))
		for k, v := range schemaProps {
			fieldSchema[k] = v
		}
		
		properties[jsonName] = fieldSchema
		
		// Check if field is required
		if isRequiredField(field) {
			required = append(required, jsonName)
		}
	}
	
	schema := map[string]interface{}{
		"type":       "object",
		"properties": properties,
	}
	
	if len(required) > 0 {
		schema["required"] = required
	}
	
	return schema
}

// generateScalarSchema creates schema for non-struct types
func generateScalarSchema(t reflect.Type) map[string]interface{} {
	return map[string]interface{}{
		"type": jsonTypeFromGo(t),
	}
}

// generateFieldSchema creates schema for individual struct fields
func generateFieldSchema(field reflect.StructField) map[string]interface{} {
	// Create a new stack for this field traversal
	stack := make([]reflect.Type, 0)
	return generateFieldSchemaWithStack(field, stack)
}

// generateFieldSchemaWithStack creates schema for individual struct fields with circular reference detection
func generateFieldSchemaWithStack(field reflect.StructField, stack []reflect.Type) map[string]interface{} {
	schema := map[string]interface{}{}
	
	// Get JSON type first
	jsonType := jsonTypeFromGo(field.Type)
	if jsonType != "" {
		schema["type"] = jsonType
	}
	
	// Special case: []byte should have format "binary"
	if field.Type.Kind() == reflect.Slice && field.Type.Elem().Kind() == reflect.Uint8 {
		schema["format"] = "binary"
	}
	
	// Special case: interface{} should not have restrictive type
	if field.Type.Kind() == reflect.Interface {
		// Don't set a type constraint for interface{}
		delete(schema, "type")
	}
	
	// Add description if available
	if desc := getFieldDescription(field); desc != "" {
		schema["description"] = desc
	}
	
	// Handle nested structs
	if field.Type.Kind() == reflect.Struct {
		schema = generateStructSchemaWithStack(field.Type, stack)
	} else if field.Type.Kind() == reflect.Ptr && field.Type.Elem().Kind() == reflect.Struct {
		schema = generateStructSchemaWithStack(field.Type.Elem(), stack)
	} else if field.Type.Kind() == reflect.Slice && field.Type.Elem().Kind() != reflect.Uint8 {
		// Handle slices (except []byte which is handled above)
		schema["type"] = "array"
		elemType := field.Type.Elem()
		if elemType.Kind() == reflect.Struct {
			schema["items"] = generateStructSchemaWithStack(elemType, stack)
		} else if elemType.Kind() == reflect.Ptr && elemType.Elem().Kind() == reflect.Struct {
			schema["items"] = generateStructSchemaWithStack(elemType.Elem(), stack)
		} else {
			itemType := jsonTypeFromGo(elemType)
			if itemType != "" {
				schema["items"] = map[string]interface{}{
					"type": itemType,
				}
			} else {
				schema["items"] = map[string]interface{}{}
			}
		}
	} else if field.Type.Kind() == reflect.Map {
		schema["type"] = "object"
		valueType := field.Type.Elem()
		if valueType.Kind() == reflect.Interface {
			// For map[string]interface{}, allow any values
			schema["additionalProperties"] = true
		} else {
			valueJsonType := jsonTypeFromGo(valueType)
			if valueJsonType != "" {
				schema["additionalProperties"] = map[string]interface{}{
					"type": valueJsonType,
				}
			} else {
				schema["additionalProperties"] = true
			}
		}
	}
	
	return schema
}

// getJSONFieldName extracts the JSON field name from struct tags
func getJSONFieldName(field reflect.StructField) string {
	jsonTag := field.Tag.Get("json")
	if jsonTag == "" {
		// No json tag, return empty string to signal field should be ignored
		return ""
	}
	
	// Split on comma to get field name only
	parts := strings.Split(jsonTag, ",")
	fieldName := parts[0]
	
	// If field name is explicitly set to "-", exclude it
	if fieldName == "-" {
		return "-"
	}
	
	// If no explicit field name, use the struct field name
	if fieldName == "" {
		return strings.ToLower(field.Name)
	}
	
	return fieldName
}

// isRequiredField determines if a field is required based on struct tags
func isRequiredField(field reflect.StructField) bool {
	jsonTag := field.Tag.Get("json")
	schemaTag := field.Tag.Get("jsonschema")
	
	// Field is optional if it has "omitempty" in json tag
	if strings.Contains(jsonTag, "omitempty") {
		return false
	}
	
	// Field is required only if explicitly marked in jsonschema tag
	if strings.Contains(schemaTag, "required") {
		return true
	}
	
	// Default: fields are optional unless explicitly required
	return false
}

// getFieldDescription extracts description from jsonschema tag
func getFieldDescription(field reflect.StructField) string {
	schemaTag := field.Tag.Get("jsonschema")
	if schemaTag == "" {
		return ""
	}
	
	// CRAWL stub: Simple description extraction
	// Look for description=... pattern
	parts := strings.Split(schemaTag, ",")
	for _, part := range parts {
		part = strings.TrimSpace(part)
		if strings.HasPrefix(part, "description=") {
			return strings.TrimPrefix(part, "description=")
		}
	}
	
	return ""
}

// jsonTypeFromGo maps Go types to JSON schema types
func jsonTypeFromGo(t reflect.Type) string {
	if t == nil {
		return "null"
	}
	
	// Handle pointers by getting the underlying type
	if t.Kind() == reflect.Ptr {
		t = t.Elem()
	}
	
	// Special case: []byte should be string (base64 encoded)
	if t.Kind() == reflect.Slice && t.Elem().Kind() == reflect.Uint8 {
		return "string"
	}
	
	switch t.Kind() {
	case reflect.String:
		return "string"
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		return "integer"
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return "integer"
	case reflect.Float32, reflect.Float64:
		return "number"
	case reflect.Bool:
		return "boolean"
	case reflect.Slice, reflect.Array:
		return "array"
	case reflect.Struct, reflect.Map:
		return "object"
	case reflect.Interface:
		// interface{} should not have a restrictive type
		return ""
	default:
		// Default to string for unknown types
		return "string"
	}
}

// mapGoTypeToJSONType is an alias for jsonTypeFromGo for backward compatibility with tests
func mapGoTypeToJSONType(t reflect.Type) string {
	if t == nil {
		return "null"
	}
	return jsonTypeFromGo(t)
}

// parseSchemaTag parses jsonschema struct tag into schema properties
func parseSchemaTag(tag string) map[string]interface{} {
	properties := make(map[string]interface{})
	
	if tag == "" {
		return properties
	}
	
	// Handle enum specially since it can contain commas in its value
	if enumStart := strings.Index(tag, "enum="); enumStart != -1 {
		// Find the enum value by looking for the next constraint or end of string
		enumPart := tag[enumStart:]
		enumEnd := len(enumPart)
		
		// Look for the next constraint (something that looks like "key=")
		for i := 5; i < len(enumPart)-2; i++ { // Start after "enum="
			if enumPart[i] == ',' && i+1 < len(enumPart) {
				// Check if what follows looks like a key=value pattern
				remaining := enumPart[i+1:]
				if nextEq := strings.Index(remaining, "="); nextEq != -1 {
					// Check if there's a valid constraint key before the =
					potentialKey := strings.TrimSpace(remaining[:nextEq])
					validKeys := []string{"minimum", "maximum", "minLength", "maxLength", "minItems", "maxItems", "description", "title", "pattern", "format"}
					for _, validKey := range validKeys {
						if potentialKey == validKey {
							enumEnd = i
							break
						}
					}
					if enumEnd < len(enumPart) {
						break
					}
				}
			}
		}
		
		enumValue := enumPart[5:enumEnd] // Skip "enum="
		enumValues := strings.Split(enumValue, ",")
		enumInterface := make([]interface{}, len(enumValues))
		for i, v := range enumValues {
			enumInterface[i] = strings.TrimSpace(v)
		}
		properties["enum"] = enumInterface
		
		// Remove enum part from tag for normal processing
		beforeEnum := tag[:enumStart]
		afterEnum := ""
		if enumStart+enumEnd < len(tag) {
			afterEnum = tag[enumStart+enumEnd:]
		}
		tag = strings.Trim(beforeEnum+afterEnum, ",")
	}
	
	// Process remaining parts normally
	parts := strings.Split(tag, ",")
	for _, part := range parts {
		part = strings.TrimSpace(part)
		
		if part == "" || part == "required" {
			// Empty parts or required is handled elsewhere
			continue
		}
		
		if strings.Contains(part, "=") {
			kv := strings.SplitN(part, "=", 2)
			if len(kv) == 2 {
				key := strings.TrimSpace(kv[0])
				value := strings.TrimSpace(kv[1])
				
				// Skip enum since we handled it above
				if key == "enum" {
					continue
				}
				
				// Parse different value types
				switch key {
				case "minimum", "maximum":
					// Try to parse as number
					if val, err := parseNumericValue(value); err == nil {
						properties[key] = val
					}
				case "minLength", "maxLength", "minItems", "maxItems":
					// Try to parse as integer
					if val, err := strconv.Atoi(value); err == nil {
						properties[key] = val
					}
				case "description", "title", "pattern", "format":
					// String values
					properties[key] = value
				default:
					// Default to string
					properties[key] = value
				}
			}
		}
	}
	
	return properties
}

// parseNumericValue attempts to parse a string as a numeric value
func parseNumericValue(s string) (interface{}, error) {
	// Try integer first - return as int if it fits
	if val, err := strconv.ParseInt(s, 10, 64); err == nil {
		if val >= int64(int(^uint(0) >> 1)) * -1 && val <= int64(int(^uint(0) >> 1)) {
			return int(val), nil
		}
		return val, nil
	}
	// Try float
	if val, err := strconv.ParseFloat(s, 64); err == nil {
		return val, nil
	}
	return nil, fmt.Errorf("not a number: %s", s)
}