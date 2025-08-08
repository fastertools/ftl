# Composing Polyglot Servers

In this tutorial, you'll experience FTL's killer feature: seamlessly composing tools written in different programming languages into a single MCP server.

## What You'll Build

Starting from your first project, you'll:
- Add a second tool written in Python
- See how Rust and Python tools work together
- Understand how the WebAssembly Component Model enables language interoperability
- Build a practical multi-language MCP server

## Prerequisites

- Complete the [Your First FTL Project](./first-project.md) tutorial
- Python 3.10+ installed
- Your `my-first-project` from the previous tutorial

## Step 1: Add a Python Tool

Let's add a tool that complements our Rust greeting tool. We'll create a Python tool that generates random facts:

```bash
cd my-first-project
ftl add random-fact --language python
```

This creates a new Python component alongside your existing Rust tool:

```bash
tree components/
```

You should see:

```
components/
├── hello-world/          # Your Rust tool
│   ├── Cargo.toml
│   ├── src/lib.rs
│   └── README.md
└── random-fact/          # New Python tool
    ├── pyproject.toml
    ├── src/
    │   └── __init__.py
    └── README.md
```

## Step 2: Implement the Python Tool

Open `components/random-fact/src/__init__.py` and replace the generated code:

```python
from ftl_sdk import tool, ToolResponse
import random

# A collection of interesting facts
FACTS = [
    "Honey never spoils. Archaeologists have found edible honey in ancient Egyptian tombs.",
    "A group of flamingos is called a 'flamboyance'.",
    "Octopuses have three hearts and blue blood.",
    "The shortest war in history lasted only 38-45 minutes between Britain and Zanzibar in 1896.",
    "Bananas are berries, but strawberries aren't.",
    "A single cloud can weigh more than a million pounds.",
    "The human brain uses about 20% of the body's total energy.",
    "There are more possible games of chess than atoms in the observable universe."
]

@tool
def get_random_fact(category: str = "general") -> ToolResponse:
    """Get a random interesting fact.
    
    Args:
        category: The category of fact (currently only 'general' is supported)
    
    Returns:
        A random fact as a string
    """
    fact = random.choice(FACTS)
    return ToolResponse.ok(f"🎯 Fun Fact: {fact}")

@tool
def get_fact_count() -> ToolResponse:
    """Get the total number of available facts.
    
    Returns:
        The number of facts in the database
    """
    count = len(FACTS)
    return ToolResponse.ok(f"📚 I know {count} interesting facts!")
```

## Step 3: Build Both Languages

Now let's build our polyglot project:

```bash
ftl build
```

Watch the output - you'll see both languages being compiled:

```
🔄 Building component: hello-world (rust)
   Compiling hello-world v0.1.0
   ✅ Built hello-world.wasm

🔄 Building component: random-fact (python)  
   Building Python component with componentize-py
   ✅ Built random-fact.wasm

✅ Build complete! 3 tools available:
   - hello-world/say_hello
   - random-fact/get_random_fact  
   - random-fact/get_fact_count
```

## Step 4: Test the Polyglot Server

Start your server:

```bash
ftl up
```

Now you have an MCP server with tools written in two different languages! Let's test both:

### Test the Rust tool:
```bash
curl -X POST http://localhost:3000/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "hello-world/say_hello",
    "arguments": {
      "name": "Polyglot Developer"
    }
  }'
```

### Test the Python tool:
```bash
curl -X POST http://localhost:3000/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "random-fact/get_random_fact",
    "arguments": {}
  }'
```

### Test the fact count:
```bash
curl -X POST http://localhost:3000/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "random-fact/get_fact_count",
    "arguments": {}
  }'
```

## Step 5: Add Even More Languages

Let's add a Go tool for mathematical operations:

```bash
ftl add math-utils --language go
```

Edit `components/math-utils/main.go`:

```go
//go:build wasip1

package main

import (
    "context"
    "encoding/json"
    "log"

    "go.bytecodealliance.org/cm"
    "go.bytecodealliance.org/wasm"
)

//go:generate wit-bindgen tiny-go wit --out-dir=gen
func init() {
    a := &MathUtilsImpl{}
    wasm.SetExportsWorldMath(a)
}

type MathUtilsImpl struct{}

type AddRequest struct {
    A float64 `json:"a"`
    B float64 `json:"b"`
}

type MultiplyRequest struct {
    Numbers []float64 `json:"numbers"`
}

func (m *MathUtilsImpl) Add(ctx context.Context, args string) cm.Result[string, string] {
    var req AddRequest
    if err := json.Unmarshal([]byte(args), &req); err != nil {
        return cm.Err[string]("Invalid arguments: " + err.Error())
    }
    
    result := req.A + req.B
    response := map[string]interface{}{
        "result": result,
        "operation": "addition",
    }
    
    jsonBytes, _ := json.Marshal(response)
    return cm.OK[string](string(jsonBytes))
}

func (m *MathUtilsImpl) Multiply(ctx context.Context, args string) cm.Result[string, string] {
    var req MultiplyRequest
    if err := json.Unmarshal([]byte(args), &req); err != nil {
        return cm.Err[string]("Invalid arguments: " + err.Error())
    }
    
    result := 1.0
    for _, num := range req.Numbers {
        result *= num
    }
    
    response := map[string]interface{}{
        "result": result,
        "operation": "multiplication",
        "factors_count": len(req.Numbers),
    }
    
    jsonBytes, _ := json.Marshal(response)
    return cm.OK[string](string(jsonBytes))
}

func main() {
    log.Println("Math utils component initialized")
}
```

## Step 6: Build All Three Languages

```bash
ftl build
```

Now you'll see all three languages being compiled:

```
🔄 Building component: hello-world (rust)
🔄 Building component: random-fact (python)
🔄 Building component: math-utils (go)

✅ Build complete! 5 tools available:
   - hello-world/say_hello
   - random-fact/get_random_fact
   - random-fact/get_fact_count
   - math-utils/add
   - math-utils/multiply
```

## Step 7: Test the Complete Polyglot Server

```bash
ftl up
```

Test the Go tool:

```bash
curl -X POST http://localhost:3000/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "math-utils/add",
    "arguments": {
      "a": 15,
      "b": 27
    }
  }'
```

## Step 8: Create a Tool Composition

Create a simple script that uses all three languages together:

```bash
# test-polyglot.sh
#!/bin/bash

echo "🚀 Testing our polyglot MCP server!"
echo

echo "1. Getting a greeting (Rust):"
curl -s -X POST http://localhost:3000/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name": "hello-world/say_hello", "arguments": {"name": "Polyglot Master"}}' | jq -r '.content[0].text'

echo
echo "2. Getting a random fact (Python):"
curl -s -X POST http://localhost:3000/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name": "random-fact/get_random_fact", "arguments": {}}' | jq -r '.content[0].text'

echo  
echo "3. Doing some math (Go):"
curl -s -X POST http://localhost:3000/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name": "math-utils/multiply", "arguments": {"numbers": [3, 7, 2]}}' | jq -r '.content[0].text'

echo
echo "🎉 All three languages working together seamlessly!"
```

Run it:

```bash
chmod +x test-polyglot.sh
./test-polyglot.sh
```

## What's Happening Under the Hood?

This seamless language interoperability is powered by:

1. **WebAssembly Component Model**: Each tool compiles to a standardized WASM component interface
2. **Universal Protocol**: All tools speak the same MCP protocol regardless of implementation language
3. **Spin Framework**: Orchestrates and routes requests between components
4. **FTL Runtime**: Handles lifecycle, security, and communication

## Key Insights

🔍 **Language Choice is Tactical**: Pick the best language for each tool:
- Rust: System tools, performance-critical operations
- Python: Data science, AI/ML, rapid prototyping  
- Go: Network services, concurrent operations
- TypeScript: Web APIs, JSON processing

🔄 **No Performance Penalties**: WASM compilation means no interpretation overhead

🛡️ **Sandboxed Security**: Each tool runs in its own isolated environment

📦 **Simple Deployment**: One server, multiple languages, single deployment unit

## What You've Learned

Congratulations! You've just:

✅ **Built a polyglot MCP server** with tools in Rust, Python, and Go  
✅ **Experienced true language interoperability** via WebAssembly  
✅ **Understood the Component Model's role** in language integration  
✅ **Composed tools tactically** by choosing the right language for each job  
✅ **Deployed multiple languages** as a single, cohesive server  

## Next Steps

Now that you've mastered polyglot composition:

- **Understand the Architecture**: Read [Core Concepts](../core-concepts/) to learn how the Component Model works
- **Solve Real Problems**: Check [How-to Guides](../guides/) for practical recipes
- **Master the APIs**: Explore [SDK Reference](../sdk-reference/) for each language
- **Get Inspired**: Browse [Examples](../../examples/) for advanced patterns

You're now ready to build sophisticated MCP servers that leverage the strengths of multiple programming languages!