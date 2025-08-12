package ftl

import (
	"reflect"
	"testing"
)

// TestGenerateSchema_BasicTypes tests schema generation for basic Go types
func TestGenerateSchema_BasicTypes(t *testing.T) {
	type BasicTypes struct {
		StringField  string  `json:"string_field"`
		IntField     int     `json:"int_field"`
		Int32Field   int32   `json:"int32_field"`
		Int64Field   int64   `json:"int64_field"`
		FloatField   float32 `json:"float_field"`
		DoubleField  float64 `json:"double_field"`
		BoolField    bool    `json:"bool_field"`
		BytesField   []byte  `json:"bytes_field"`
	}

	schema := generateSchema[BasicTypes]()

	// Verify top-level schema structure
	if schema["type"] != "object" {
		t.Errorf("Expected schema type 'object', got %v", schema["type"])
	}

	properties, ok := schema["properties"].(map[string]interface{})
	if !ok {
		t.Fatal("Schema should have properties as map")
	}

	// Test string field
	stringField := properties["string_field"].(map[string]interface{})
	if stringField["type"] != "string" {
		t.Errorf("String field should have type 'string', got %v", stringField["type"])
	}

	// Test integer fields
	intField := properties["int_field"].(map[string]interface{})
	if intField["type"] != "integer" {
		t.Errorf("Int field should have type 'integer', got %v", intField["type"])
	}

	// Test float fields  
	floatField := properties["float_field"].(map[string]interface{})
	if floatField["type"] != "number" {
		t.Errorf("Float field should have type 'number', got %v", floatField["type"])
	}

	// Test boolean field
	boolField := properties["bool_field"].(map[string]interface{})
	if boolField["type"] != "boolean" {
		t.Errorf("Bool field should have type 'boolean', got %v", boolField["type"])
	}

	// Test bytes field (should be string with format)
	bytesField := properties["bytes_field"].(map[string]interface{})
	if bytesField["type"] != "string" {
		t.Errorf("Bytes field should have type 'string', got %v", bytesField["type"])
	}
	if bytesField["format"] != "binary" {
		t.Errorf("Bytes field should have format 'binary', got %v", bytesField["format"])
	}
}

// TestGenerateSchema_JSONSchemaTagConstraints tests jsonschema tag parsing
func TestGenerateSchema_JSONSchemaTagConstraints(t *testing.T) {
	type ConstrainedType struct {
		RequiredField   string  `json:"required_field" jsonschema:"required,description=A required field"`
		MinMaxInt       int     `json:"min_max_int" jsonschema:"minimum=1,maximum=100"`
		MinMaxFloat     float64 `json:"min_max_float" jsonschema:"minimum=0.0,maximum=1.0"`
		PatternString   string  `json:"pattern_string" jsonschema:"pattern=^[a-zA-Z]+$"`
		MinMaxLenString string  `json:"minmaxlen_string" jsonschema:"minLength=5,maxLength=50"`
		MinMaxItems     []int   `json:"minmaxitems_array" jsonschema:"minItems=1,maxItems=10"`
		EnumField       string  `json:"enum_field" jsonschema:"enum=red,green,blue"`
	}

	schema := generateSchema[ConstrainedType]()
	properties := schema["properties"].(map[string]interface{})

	// Test required field
	required, ok := schema["required"].([]string)
	if !ok {
		t.Fatal("Schema should have required array")
	}
	
	requiredFound := false
	for _, field := range required {
		if field == "required_field" {
			requiredFound = true
			break
		}
	}
	if !requiredFound {
		t.Error("Required field should be in required array")
	}

	requiredField := properties["required_field"].(map[string]interface{})
	if requiredField["description"] != "A required field" {
		t.Errorf("Required field description should be 'A required field', got %v", requiredField["description"])
	}

	// Test integer constraints
	minMaxInt := properties["min_max_int"].(map[string]interface{})
	if minMaxInt["minimum"] != 1 {
		t.Errorf("MinMax int minimum should be 1, got %v", minMaxInt["minimum"])
	}
	if minMaxInt["maximum"] != 100 {
		t.Errorf("MinMax int maximum should be 100, got %v", minMaxInt["maximum"])
	}

	// Test float constraints
	minMaxFloat := properties["min_max_float"].(map[string]interface{})
	if minMaxFloat["minimum"] != 0.0 {
		t.Errorf("MinMax float minimum should be 0.0, got %v", minMaxFloat["minimum"])
	}
	if minMaxFloat["maximum"] != 1.0 {
		t.Errorf("MinMax float maximum should be 1.0, got %v", minMaxFloat["maximum"])
	}

	// Test pattern constraint
	patternString := properties["pattern_string"].(map[string]interface{})
	if patternString["pattern"] != "^[a-zA-Z]+$" {
		t.Errorf("Pattern string pattern should be '^[a-zA-Z]+$', got %v", patternString["pattern"])
	}

	// Test string length constraints
	minMaxLenString := properties["minmaxlen_string"].(map[string]interface{})
	if minMaxLenString["minLength"] != 5 {
		t.Errorf("MinMaxLen string minLength should be 5, got %v", minMaxLenString["minLength"])
	}
	if minMaxLenString["maxLength"] != 50 {
		t.Errorf("MinMaxLen string maxLength should be 50, got %v", minMaxLenString["maxLength"])
	}

	// Test array item constraints
	minMaxItems := properties["minmaxitems_array"].(map[string]interface{})
	if minMaxItems["type"] != "array" {
		t.Errorf("MinMaxItems should be array type, got %v", minMaxItems["type"])
	}
	if minMaxItems["minItems"] != 1 {
		t.Errorf("MinMaxItems minItems should be 1, got %v", minMaxItems["minItems"])
	}
	if minMaxItems["maxItems"] != 10 {
		t.Errorf("MinMaxItems maxItems should be 10, got %v", minMaxItems["maxItems"])
	}

	// Test enum constraint
	enumField := properties["enum_field"].(map[string]interface{})
	enumValues, ok := enumField["enum"].([]interface{})
	if !ok {
		t.Fatal("Enum field should have enum array")
	}
	expectedEnum := []string{"red", "green", "blue"}
	if len(enumValues) != len(expectedEnum) {
		t.Errorf("Enum should have %d values, got %d", len(expectedEnum), len(enumValues))
	}
}

// TestGenerateSchema_NestedStructs tests schema generation for nested structures
func TestGenerateSchema_NestedStructs(t *testing.T) {
	type Address struct {
		Street   string `json:"street" jsonschema:"required"`
		City     string `json:"city" jsonschema:"required"`
		Country  string `json:"country" jsonschema:"required"`
		PostCode string `json:"post_code"`
	}

	type Person struct {
		Name         string    `json:"name" jsonschema:"required"`
		Age          int       `json:"age" jsonschema:"minimum=0,maximum=150"`
		HomeAddress  Address   `json:"home_address" jsonschema:"required"`
		WorkAddress  *Address  `json:"work_address,omitempty"`
		Addresses    []Address `json:"addresses,omitempty"`
	}

	schema := generateSchema[Person]()
	properties := schema["properties"].(map[string]interface{})

	// Test nested struct field
	homeAddress := properties["home_address"].(map[string]interface{})
	if homeAddress["type"] != "object" {
		t.Errorf("HomeAddress should be object type, got %v", homeAddress["type"])
	}

	// Test nested struct properties
	homeProps, ok := homeAddress["properties"].(map[string]interface{})
	if !ok {
		t.Fatal("HomeAddress should have properties")
	}

	streetField := homeProps["street"].(map[string]interface{})
	if streetField["type"] != "string" {
		t.Errorf("Street field should be string type, got %v", streetField["type"])
	}

	// Test nested struct required fields
	homeRequired, ok := homeAddress["required"].([]string)
	if !ok {
		t.Fatal("HomeAddress should have required fields")
	}
	if len(homeRequired) != 3 { // street, city, country
		t.Errorf("HomeAddress should have 3 required fields, got %d", len(homeRequired))
	}

	// Test pointer to struct (should be same as struct but optional)
	workAddress := properties["work_address"].(map[string]interface{})
	if workAddress["type"] != "object" {
		t.Errorf("WorkAddress should be object type, got %v", workAddress["type"])
	}

	// Test array of structs
	addresses := properties["addresses"].(map[string]interface{})
	if addresses["type"] != "array" {
		t.Errorf("Addresses should be array type, got %v", addresses["type"])
	}

	addressItems, ok := addresses["items"].(map[string]interface{})
	if !ok {
		t.Fatal("Addresses array should have items schema")
	}
	if addressItems["type"] != "object" {
		t.Errorf("Address items should be object type, got %v", addressItems["type"])
	}
}

// TestGenerateSchema_ComplexTypes tests maps, interfaces, and custom types
func TestGenerateSchema_ComplexTypes(t *testing.T) {
	type CustomString string
	type CustomInt int

	type ComplexType struct {
		StringMap     map[string]string      `json:"string_map,omitempty"`
		InterfaceMap  map[string]interface{} `json:"interface_map,omitempty"`
		IntSlice      []int                  `json:"int_slice,omitempty"`
		StringSlice   []string               `json:"string_slice,omitempty"`
		CustomStr     CustomString           `json:"custom_str"`
		CustomNumber  CustomInt              `json:"custom_number"`
		AnyInterface  interface{}            `json:"any_interface,omitempty"`
	}

	schema := generateSchema[ComplexType]()
	properties := schema["properties"].(map[string]interface{})

	// Test string map
	stringMap := properties["string_map"].(map[string]interface{})
	if stringMap["type"] != "object" {
		t.Errorf("StringMap should be object type, got %v", stringMap["type"])
	}
	additionalProps, ok := stringMap["additionalProperties"].(map[string]interface{})
	if !ok {
		t.Fatal("StringMap should have additionalProperties")
	}
	if additionalProps["type"] != "string" {
		t.Errorf("StringMap additionalProperties should be string type, got %v", additionalProps["type"])
	}

	// Test interface map
	interfaceMap := properties["interface_map"].(map[string]interface{})
	if interfaceMap["type"] != "object" {
		t.Errorf("InterfaceMap should be object type, got %v", interfaceMap["type"])
	}

	// Test slices
	intSlice := properties["int_slice"].(map[string]interface{})
	if intSlice["type"] != "array" {
		t.Errorf("IntSlice should be array type, got %v", intSlice["type"])
	}
	intItems, ok := intSlice["items"].(map[string]interface{})
	if !ok {
		t.Fatal("IntSlice should have items schema")
	}
	if intItems["type"] != "integer" {
		t.Errorf("IntSlice items should be integer type, got %v", intItems["type"])
	}

	// Test custom types (should map to underlying type)
	customStr := properties["custom_str"].(map[string]interface{})
	if customStr["type"] != "string" {
		t.Errorf("CustomStr should be string type, got %v", customStr["type"])
	}

	customNumber := properties["custom_number"].(map[string]interface{})
	if customNumber["type"] != "integer" {
		t.Errorf("CustomNumber should be integer type, got %v", customNumber["type"])
	}

	// Test interface{} (should allow any type)
	anyInterface := properties["any_interface"].(map[string]interface{})
	// interface{} should not have a specific type constraint or should be "any"
	if anyInterface["type"] != nil && anyInterface["type"] != "any" {
		t.Errorf("AnyInterface should not have restrictive type, got %v", anyInterface["type"])
	}
}

// TestGenerateSchema_OmitEmptyHandling tests omitempty tag handling
func TestGenerateSchema_OmitEmptyHandling(t *testing.T) {
	type OmitEmptyType struct {
		Required    string  `json:"required" jsonschema:"required"`
		Optional    string  `json:"optional,omitempty"`
		OptionalInt int     `json:"optional_int,omitempty"`
		OptionalPtr *string `json:"optional_ptr,omitempty"`
	}

	schema := generateSchema[OmitEmptyType]()

	// Check required fields
	required, ok := schema["required"].([]string)
	if !ok {
		t.Fatal("Schema should have required array")
	}

	// Required field should be in required array
	requiredFound := false
	optionalFound := false
	for _, field := range required {
		if field == "required" {
			requiredFound = true
		}
		if field == "optional" {
			optionalFound = true
		}
	}

	if !requiredFound {
		t.Error("Required field should be in required array")
	}
	if optionalFound {
		t.Error("Optional field should not be in required array")
	}

	// All fields should still be in properties (omitempty affects required, not presence)
	properties := schema["properties"].(map[string]interface{})
	if len(properties) != 4 {
		t.Errorf("Should have 4 properties, got %d", len(properties))
	}
}

// TestParseJSONSchemaTag tests the JSON schema tag parsing function
func TestParseJSONSchemaTag(t *testing.T) {
	tests := []struct {
		tag      string
		expected map[string]interface{}
	}{
		{
			tag: "required,description=A test field",
			expected: map[string]interface{}{
				"description": "A test field",
			},
		},
		{
			tag: "minimum=0,maximum=100,description=A number field",
			expected: map[string]interface{}{
				"minimum":     0,
				"maximum":     100, 
				"description": "A number field",
			},
		},
		{
			tag: "pattern=^[a-zA-Z]+$,minLength=1,maxLength=50",
			expected: map[string]interface{}{
				"pattern":   "^[a-zA-Z]+$",
				"minLength": 1,
				"maxLength": 50,
			},
		},
		{
			tag: "enum=red,green,blue",
			expected: map[string]interface{}{
				"enum": []interface{}{"red", "green", "blue"},
			},
		},
		{
			tag: "",
			expected: map[string]interface{}{},
		},
	}

	for _, test := range tests {
		result := parseSchemaTag(test.tag)
		
		if len(result) != len(test.expected) {
			t.Errorf("For tag '%s', expected %d items, got %d", test.tag, len(test.expected), len(result))
			continue
		}

		for key, expectedValue := range test.expected {
			if actualValue, ok := result[key]; !ok {
				t.Errorf("For tag '%s', expected key '%s' not found", test.tag, key)
			} else if !compareValues(actualValue, expectedValue) {
				t.Errorf("For tag '%s', key '%s': expected '%v', got '%v'", test.tag, key, expectedValue, actualValue)
			}
		}
	}
}

// compareValues compares two values, handling slices specially
func compareValues(actual, expected interface{}) bool {
	// Handle slice comparison
	actualSlice, actualIsSlice := actual.([]interface{})
	expectedSlice, expectedIsSlice := expected.([]interface{})
	
	if actualIsSlice && expectedIsSlice {
		if len(actualSlice) != len(expectedSlice) {
			return false
		}
		for i, v := range actualSlice {
			if v != expectedSlice[i] {
				return false
			}
		}
		return true
	}
	
	// Regular comparison
	return actual == expected
}

// TestMapGoTypeToJSONType tests Go type to JSON type mapping
func TestMapGoTypeToJSONType(t *testing.T) {
	tests := []struct {
		goType   reflect.Type
		expected string
	}{
		{reflect.TypeOf(""), "string"},
		{reflect.TypeOf(0), "integer"},
		{reflect.TypeOf(int32(0)), "integer"},
		{reflect.TypeOf(int64(0)), "integer"},
		{reflect.TypeOf(float32(0)), "number"},
		{reflect.TypeOf(float64(0)), "number"},
		{reflect.TypeOf(true), "boolean"},
		{reflect.TypeOf([]byte{}), "string"}, // bytes are base64 encoded strings
		{reflect.TypeOf([]int{}), "array"},
		{reflect.TypeOf(map[string]interface{}{}), "object"},
	}

	for _, test := range tests {
		result := mapGoTypeToJSONType(test.goType)
		if result != test.expected {
			t.Errorf("For Go type %v, expected JSON type '%s', got '%s'", test.goType, test.expected, result)
		}
	}
}

// TestGenerateSchema_EdgeCases tests edge cases and error conditions
func TestGenerateSchema_EdgeCases(t *testing.T) {
	// Test empty struct
	type EmptyStruct struct{}
	
	schema := generateSchema[EmptyStruct]()
	if schema["type"] != "object" {
		t.Errorf("Empty struct should still be object type, got %v", schema["type"])
	}
	
	properties := schema["properties"].(map[string]interface{})
	if len(properties) != 0 {
		t.Errorf("Empty struct should have 0 properties, got %d", len(properties))
	}

	// Test struct with unexported fields (should be ignored)
	type MixedStruct struct {
		Public    string `json:"public"`
		private   string `json:"private"` // Should be ignored
		NoJSON    string // Should be ignored (no json tag)
		SkipField string `json:"-"`      // Should be ignored (json:"-")
	}

	schema = generateSchema[MixedStruct]()
	properties = schema["properties"].(map[string]interface{})
	
	if len(properties) != 1 { // Only "public" should be included
		t.Errorf("MixedStruct should have 1 property, got %d", len(properties))
	}

	if _, ok := properties["public"]; !ok {
		t.Error("Public field should be in properties")
	}
	if _, ok := properties["private"]; ok {
		t.Error("Private field should not be in properties")
	}
	if _, ok := properties["NoJSON"]; ok {
		t.Error("Field without json tag should not be in properties")
	}
	if _, ok := properties["SkipField"]; ok {
		t.Error("Field with json:\"-\" should not be in properties")
	}
}

// TestGenerateSchema_Recursive tests handling of recursive/circular structures  
func TestGenerateSchema_Recursive(t *testing.T) {
	type Node struct {
		Value    string `json:"value"`
		Children []*Node `json:"children,omitempty"`
		Parent   *Node   `json:"parent,omitempty"`
	}

	// This should not cause infinite recursion
	// Implementation should either:
	// 1. Detect cycles and use $ref
	// 2. Limit recursion depth
	// 3. Handle pointers specially
	schema := generateSchema[Node]()

	if schema["type"] != "object" {
		t.Errorf("Node should be object type, got %v", schema["type"])
	}

	properties := schema["properties"].(map[string]interface{})
	if len(properties) != 3 {
		t.Errorf("Node should have 3 properties, got %d", len(properties))
	}

	// Test that recursive fields are handled without infinite loops
	// (specific behavior depends on implementation approach)
	if _, ok := properties["children"]; !ok {
		t.Error("Children field should be present")
	}
	if _, ok := properties["parent"]; !ok {
		t.Error("Parent field should be present")
	}
}