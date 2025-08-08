# The Component Model

The WebAssembly Component Model is the secret sauce that makes FTL's polyglot capabilities possible. It's like creating universal adapters for different electronic plugs - a Go "plug" can fit into a Rust "socket" seamlessly.

## The Language Interoperability Problem

Traditionally, making different programming languages work together has been painful:

### The Old Way: Foreign Function Interfaces (FFI)

```c
// C library
int add(int a, int b) { return a + b; }
```

```rust
// Rust calling C
extern "C" {
    fn add(a: i32, b: i32) -> i32;
}

unsafe {
    let result = add(5, 3); // Requires unsafe code
}
```

```python  
# Python calling C
import ctypes
lib = ctypes.CDLL('./libmath.so')  # Platform-specific
lib.add.argtypes = (ctypes.c_int, ctypes.c_int)
lib.add.restype = ctypes.c_int
result = lib.add(5, 3)
```

**Problems with FFI:**
- ❌ **Platform-specific**: Different approaches for each OS
- ❌ **Unsafe**: Easy to cause crashes with type mismatches  
- ❌ **Complex**: Manual memory management and marshaling
- ❌ **Fragile**: ABI compatibility issues across versions
- ❌ **Limited Types**: Only simple types work reliably

## The Component Model Solution

The WebAssembly Component Model defines a **standard interface layer** that all languages can use:

### Universal Interface Definition

Instead of language-specific bindings, you define interfaces in **WIT (WebAssembly Interface Types)**:

```wit
// math.wit - Universal interface definition
package example:math@1.0.0

interface operations {
  add: func(a: s32, b: s32) -> s32
  multiply: func(numbers: list<f64>) -> f64
  
  record point {
    x: f64,
    y: f64,
  }
  
  calculate-distance: func(p1: point, p2: point) -> f64
}

world math-tools {
  export operations
}
```

### Language Implementations

Now each language implements this interface naturally:

```rust
// Rust implementation
use example::math::operations::*;

struct MathToolsImpl;

impl Guest for MathToolsImpl {
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    
    fn multiply(numbers: Vec<f64>) -> f64 {
        numbers.iter().product()
    }
    
    fn calculate_distance(p1: Point, p2: Point) -> f64 {
        ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt()
    }
}
```

```python
# Python implementation
from example.math import operations

class MathTools:
    def add(self, a: int, b: int) -> int:
        return a + b
    
    def multiply(self, numbers: list[float]) -> float:
        result = 1.0
        for num in numbers:
            result *= num
        return result
    
    def calculate_distance(self, p1: operations.Point, p2: operations.Point) -> float:
        import math
        return math.sqrt((p2.x - p1.x)**2 + (p2.y - p1.y)**2)
```

```go
// Go implementation  
package main

import (
    "math"
    "example.com/math/gen"
)

type MathToolsImpl struct{}

func (m *MathToolsImpl) Add(a, b int32) int32 {
    return a + b
}

func (m *MathToolsImpl) Multiply(numbers []float64) float64 {
    result := 1.0
    for _, num := range numbers {
        result *= num
    }
    return result
}

func (m *MathToolsImpl) CalculateDistance(p1, p2 gen.Point) float64 {
    dx := p2.X - p1.X
    dy := p2.Y - p1.Y
    return math.Sqrt(dx*dx + dy*dy)
}
```

## How It Works: The Magic Beneath

### 1. Canonical ABI

The Component Model defines a **Canonical ABI** (Application Binary Interface) that standardizes:

```
High-Level Type    →    Canonical ABI    →    WASM Core Types
─────────────────────────────────────────────────────────────
string            →     (ptr, len)       →    (i32, i32)  
list<f64>         →     (ptr, len)       →    (i32, i32)
record { x, y }   →     (x, y)           →    (f64, f64)
result<T, E>      →     (tag, value)     →    (i32, ...)
```

### 2. Interface Adapters

Each component gets automatically generated adapters:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Rust Tool     │    │   Python Tool   │    │    Go Tool      │
│                 │    │                 │    │                 │
│  ┌───────────┐  │    │  ┌───────────┐  │    │  ┌───────────┐  │
│  │   Your    │  │    │  │   Your    │  │    │  │   Your    │  │
│  │   Code    │  │    │  │   Code    │  │    │  │   Code    │  │
│  └───────────┘  │    │  └───────────┘  │    │  └───────────┘  │
│         │        │    │         │       │    │         │       │
│  ┌───────────┐  │    │  ┌───────────┐  │    │  ┌───────────┐  │
│  │ Generated │  │    │  │ Generated │  │    │  │ Generated │  │
│  │  Adapter  │  │    │  │  Adapter  │  │    │  │  Adapter  │  │
│  └───────────┘  │    │  └───────────┘  │    │  └───────────┘  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                        │                        │
         └────────────────────────┼────────────────────────┘
                                  │
                 ┌─────────────────────────────┐
                 │   Canonical ABI Interface  │
                 └─────────────────────────────┘
```

### 3. Type Safety Across Languages

The Component Model provides **strong type safety**:

```wit
// If the interface says:
process-user: func(user: user-record) -> result<string, error>

record user-record {
  id: u32,
  name: string,
  email: string,
}

variant error {
  invalid-email(string),
  user-not-found,
  database-error(string),
}
```

Then **every language** gets the same strong typing:

```rust
// Rust gets proper Result types
fn process_user(user: UserRecord) -> Result<String, Error> { ... }
```

```python
# Python gets dataclasses and proper error handling  
def process_user(user: UserRecord) -> Result[str, Error]: ...
```

```go
// Go gets proper struct types and error returns
func ProcessUser(user UserRecord) (string, error) { ... }
```

## Real-World Example: FTL Tools

Let's see how this works in practice with FTL tools:

### 1. Tool Interface Definition

FTL automatically generates component interfaces from your tool signatures:

```rust
// Your Rust tool
#[tool]
pub fn analyze_text(content: String, options: AnalysisOptions) -> AnalysisResult {
    // Your implementation
}

#[derive(Serialize, Deserialize)]
pub struct AnalysisOptions {
    pub sentiment: bool,
    pub keywords: bool,
    pub language: Option<String>,
}
```

This automatically becomes:

```wit
// Generated WIT interface
interface text-analysis {
  record analysis-options {
    sentiment: bool,
    keywords: bool,  
    language: option<string>,
  }
  
  record analysis-result {
    sentiment-score: option<f64>,
    keywords: list<string>,
    detected-language: option<string>,
  }
  
  analyze-text: func(content: string, options: analysis-options) -> analysis-result
}
```

### 2. Cross-Language Composition

Now a Python tool can call your Rust tool seamlessly:

```python
# Python tool calling Rust tool
from ftl_sdk import tool, call_tool

@tool  
def summarize_with_analysis(document: str) -> dict:
    # Call the Rust text analysis tool
    analysis = call_tool("text-analyzer/analyze_text", {
        "content": document,
        "options": {
            "sentiment": True,
            "keywords": True,
            "language": None
        }
    })
    
    # Use the results in Python
    if analysis.sentiment_score and analysis.sentiment_score > 0.5:
        tone = "positive"
    else:
        tone = "negative" 
        
    return {
        "summary": generate_summary(document),  # Python logic
        "tone": tone,
        "keywords": analysis.keywords,
        "language": analysis.detected_language
    }
```

### 3. Type Safety Guarantees

The Component Model ensures:

- ✅ **Compile-time checks**: Wrong types are caught at build time
- ✅ **Runtime safety**: No buffer overflows or memory corruption  
- ✅ **Interface versioning**: Breaking changes are detected
- ✅ **Automatic marshaling**: No manual serialization needed

## Benefits in Action

### Traditional Multi-Language Setup
```bash
# Install multiple runtimes
brew install python rust go node
pip install requests numpy pandas
cargo install serde tokio
npm install axios lodash
go mod download

# Coordinate between languages
curl http://rust-service:8080/api/analyze \
  -d @data.json | \
python process.py | \
node format.js | \
go run summary.go
```

### FTL Component Model Approach
```bash
# Single runtime, multiple languages
ftl add text-analyzer --language rust    # Rust for performance
ftl add data-processor --language python # Python for data science  
ftl add formatter --language node        # Node for JSON processing
ftl add summarizer --language go         # Go for concurrency

ftl build  # All components work together
ftl up     # Single server, all languages
```

## Advanced Component Model Features

### Resource Management
```wit
// Shared resources across components
resource database-connection {
  constructor(url: string) -> result<database-connection, error>
  query: func(sql: string) -> result<list<record>, error>
  close: func()
}
```

### Stream Processing
```wit
// Streaming interfaces for large data
interface stream-processor {
  process-stream: func(input: stream<bytes>) -> stream<result<string, error>>
}
```

### Async Operations
```wit
// Future/Promise-like async operations
interface async-operations {
  fetch-data: func(url: string) -> future<result<bytes, http-error>>  
}
```

## Component Model vs Alternatives

| Approach | Type Safety | Performance | Complexity | Universality |
|----------|-------------|-------------|------------|--------------|
| FFI | ❌ Manual | ✅ Native | ❌ High | ❌ Platform-specific |
| HTTP APIs | ❌ Runtime only | ❌ Network overhead | ✅ Simple | ✅ Universal |
| Shared Libraries | ❌ ABI fragile | ✅ Native | ❌ High | ❌ Platform-specific |
| Component Model | ✅ Compile-time | ✅ Near-native | ✅ Generated | ✅ Universal |

## The Future of Composition

The Component Model enables new patterns:

### Capability-Based Security
```wit
// Components request only needed capabilities
world secure-tool {
  import network: network-interface
  import filesystem: readonly-filesystem  // Read-only access
  export tool: tool-interface
}
```

### Dynamic Composition
```bash
# Runtime composition without recompilation
ftl compose \
  --pipeline text-analyzer,sentiment-scorer,report-generator \
  --input document.txt \
  --output report.json
```

### Component Registries
```bash
# Discover and use community components
ftl component search text-processing
ftl component install @community/advanced-nlp:1.2.0
ftl add my-tool --use @community/advanced-nlp
```

## Learning More

- **Architecture**: See how components fit into FTL's overall design in [FTL Architecture](./architecture.md)
- **Implementation**: Learn the development workflow in [Project Lifecycle](./lifecycle.md)
- **Practice**: Try the polyglot tutorial in [Composing Polyglot Servers](../getting-started/polyglot-composition.md)

The WebAssembly Component Model transforms language interoperability from a painful integration problem into a seamless composition opportunity. It's the foundation that makes FTL's "write each tool in the best language for the job" philosophy practical and powerful.