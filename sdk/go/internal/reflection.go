// Package internal provides reflection utilities for the FTL Go SDK V3.
// This package is internal and not part of the public API.
package internal

import (
	"reflect"
	"strconv"
	"strings"
)

// TypeInfo contains reflection information about a Go type
type TypeInfo struct {
	Type         reflect.Type
	IsStruct     bool
	IsSlice      bool
	IsPointer    bool
	ElementType  reflect.Type // For slices and pointers
	Fields       []FieldInfo  // For structs
	JSONTypeName string
}

// FieldInfo contains information about a struct field
type FieldInfo struct {
	Name         string
	Type         reflect.Type
	JSONName     string
	JSONOmitEmpty bool
	JSONSkip     bool
	SchemaTag    string
	Required     bool
	Description  string
	Constraints  map[string]interface{}
}

// GetTypeInfo analyzes a Go type and returns structured information
func GetTypeInfo(t reflect.Type) TypeInfo {
	info := TypeInfo{
		Type:         t,
		JSONTypeName: GetJSONType(t),
	}
	
	// Handle pointers
	if t.Kind() == reflect.Ptr {
		info.IsPointer = true
		info.ElementType = t.Elem()
		t = t.Elem() // Work with the underlying type
	}
	
	// Handle slices
	if t.Kind() == reflect.Slice {
		info.IsSlice = true
		info.ElementType = t.Elem()
	}
	
	// Handle structs
	if t.Kind() == reflect.Struct {
		info.IsStruct = true
		info.Fields = getStructFields(t)
	}
	
	return info
}

// getStructFields extracts field information from a struct type
func getStructFields(t reflect.Type) []FieldInfo {
	var fields []FieldInfo
	
	for i := 0; i < t.NumField(); i++ {
		field := t.Field(i)
		
		// Skip unexported fields
		if !field.IsExported() {
			continue
		}
		
		fieldInfo := FieldInfo{
			Name: field.Name,
			Type: field.Type,
		}
		
		// Parse JSON tag
		parseJSONTag(field, &fieldInfo)
		
		// Parse jsonschema tag  
		parseSchemaTag(field, &fieldInfo)
		
		// Skip fields marked as "-"
		if fieldInfo.JSONSkip {
			continue
		}
		
		fields = append(fields, fieldInfo)
	}
	
	return fields
}

// parseJSONTag parses the `json` struct tag
func parseJSONTag(field reflect.StructField, info *FieldInfo) {
	jsonTag := field.Tag.Get("json")
	if jsonTag == "" {
		// Default to lowercase field name
		info.JSONName = strings.ToLower(field.Name)
		return
	}
	
	// Handle "-" (skip field)
	if jsonTag == "-" {
		info.JSONSkip = true
		return
	}
	
	// Split on comma: "fieldname,omitempty,string"
	parts := strings.Split(jsonTag, ",")
	
	// First part is the field name
	if parts[0] != "" {
		info.JSONName = parts[0]
	} else {
		info.JSONName = strings.ToLower(field.Name)
	}
	
	// Check for omitempty
	for _, part := range parts[1:] {
		if part == "omitempty" {
			info.JSONOmitEmpty = true
			break
		}
	}
}

// parseSchemaTag parses the `jsonschema` struct tag
func parseSchemaTag(field reflect.StructField, info *FieldInfo) {
	schemaTag := field.Tag.Get("jsonschema")
	if schemaTag == "" {
		return
	}
	
	info.SchemaTag = schemaTag
	info.Constraints = make(map[string]interface{})
	
	// Split on comma: "required,description=...,minimum=1"
	parts := strings.Split(schemaTag, ",")
	
	for _, part := range parts {
		part = strings.TrimSpace(part)
		
		if part == "required" {
			info.Required = true
			continue
		}
		
		// Handle key=value pairs
		if kv := strings.SplitN(part, "=", 2); len(kv) == 2 {
			key := strings.TrimSpace(kv[0])
			value := strings.TrimSpace(kv[1])
			
			if key == "description" {
				info.Description = value
			} else {
				// Try to parse numeric values
				if parsedValue := parseConstraintValue(value); parsedValue != nil {
					info.Constraints[key] = parsedValue
				} else {
					info.Constraints[key] = value
				}
			}
		}
	}
}

// parseConstraintValue attempts to parse string values into appropriate types
func parseConstraintValue(value string) interface{} {
	// Try integer
	if intVal, err := strconv.Atoi(value); err == nil {
		return intVal
	}
	
	// Try float
	if floatVal, err := strconv.ParseFloat(value, 64); err == nil {
		return floatVal
	}
	
	// Try boolean
	if boolVal, err := strconv.ParseBool(value); err == nil {
		return boolVal
	}
	
	// Return as string if no other type matches
	return nil // Caller should use original string
}

// GetJSONType returns the JSON schema type for a Go type
func GetJSONType(t reflect.Type) string {
	// Handle pointers
	if t.Kind() == reflect.Ptr {
		t = t.Elem()
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
	default:
		return "string" // Default fallback
	}
}

// GetElementType returns the element type for slices/arrays or the type itself
func GetElementType(t reflect.Type) reflect.Type {
	switch t.Kind() {
	case reflect.Slice, reflect.Array:
		return t.Elem()
	case reflect.Ptr:
		return GetElementType(t.Elem())
	default:
		return t
	}
}

// IsOptionalField determines if a field should be optional in JSON schema
func IsOptionalField(field FieldInfo) bool {
	// Explicit required tag overrides everything
	if field.Required {
		return false
	}
	
	// omitempty makes field optional
	if field.JSONOmitEmpty {
		return true
	}
	
	// Pointer types are typically optional
	if field.Type.Kind() == reflect.Ptr {
		return true
	}
	
	// Default to required (conservative approach for CRAWL phase)
	return false
}

// GetTypeName returns a human-readable name for a type (for debugging/documentation)
func GetTypeName(t reflect.Type) string {
	if t.PkgPath() != "" {
		return t.PkgPath() + "." + t.Name()
	}
	return t.Name()
}