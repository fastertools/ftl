# FTL CLI Architecture

## Design Principles

1. **Single Source of Truth**: CUE defines all configuration schemas and validation rules
2. **Fail Fast**: All user input must be validated through CUE before any operations
3. **Type Safety**: After CUE validation, use strongly-typed values throughout
4. **Clarity**: No ambiguous intermediate representations

## Data Flow

```
User Input (YAML/JSON/CUE) 
    ↓
CUE Parsing & Validation (patterns.cue)
    ↓
CUE Value (validated, typed)
    ↓
Command Operations (using CUE accessors)
    ↓
Output (Spin TOML, API calls, etc.)
```

## Configuration Pipeline

### 1. Input Stage
- Accept YAML, JSON, or CUE files
- No direct parsing to Go structs
- Pass raw bytes to CUE engine

### 2. Validation Stage
- CUE validates against `#FTLApplication` schema
- Enforces all constraints (naming, required fields, etc.)
- Returns detailed validation errors

### 3. Processing Stage
- Commands work with validated CUE values
- Use CUE path accessors to extract data
- Transform through CUE patterns when needed

### 4. Output Stage
- Generate Spin manifests via CUE patterns
- Marshal to API formats only after validation
- Maintain type safety throughout

## Package Structure

### internal/synthesis
- Pure CUE-based validation and transformation
- Defines canonical schemas in `patterns.cue`
- Handles all config format conversions

### internal/cli
- Command implementations
- Works exclusively with validated CUE values
- No direct YAML/JSON unmarshaling

### pkg/oci
- OCI registry operations
- WASM artifact handling
- Clean, focused API

## Error Handling

1. **Validation Errors**: Report CUE validation failures with context
2. **Path Errors**: Show exact CUE paths that failed
3. **Type Errors**: Catch at CUE validation, not runtime
4. **User Guidance**: Provide actionable error messages

## Security Considerations

1. All input validated before processing
2. No string concatenation for critical operations  
3. Path traversal prevention at entry points
4. Type safety enforced throughout

## Migration Path

1. Replace `pkg/types` with CUE value accessors
2. Update commands to use CUE validation pipeline
3. Remove redundant validation code
4. Consolidate error handling