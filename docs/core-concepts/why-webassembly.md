# Why WebAssembly?

FTL is built entirely on WebAssembly (WASM). This choice isn't just technical - it's fundamental to enabling FTL's unique capabilities. Let's explore why.

## The AI Tools Challenge

AI tools have unique requirements that traditional architectures struggle to address:

### Security Concerns
- **Untrusted Code**: AI tools often come from various sources and may contain bugs or malicious code
- **Isolation Needed**: Tools should not be able to access sensitive system resources or interfere with each other
- **Sandboxing Required**: Traditional processes offer weak isolation and high overhead

### Performance Requirements
- **Fast Startup**: AI workflows need tools to start quickly, not wait for language runtimes
- **Minimal Overhead**: Every millisecond counts in AI interactions
- **Resource Efficiency**: Tools should use minimal memory and CPU

### Portability Needs
- **Cross-Platform**: Tools should run identically on macOS, Linux, Windows, and cloud environments
- **Deployment Flexibility**: Same tools should work locally, in containers, and serverless environments
- **Version Consistency**: Eliminate "works on my machine" problems

## WebAssembly's Solutions

WebAssembly addresses these challenges uniquely:

### 🛡️ Security Through Sandboxing

```
┌─────────────────────────────────────────┐
│              Host System                │
├─────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│ │ WASM Tool 1 │ │ WASM Tool 2 │ │  ...    │ │
│ │   (Rust)    │ │  (Python)   │ │         │ │
│ └─────────────┘ └─────────────┘ └─────────┘ │
│          │              │                  │
│     ┌─────────────────────────────────────┐ │
│     │      WASM Runtime (Wasmtime)       │ │
│     └─────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

**Key Benefits:**
- **Memory Isolation**: Tools cannot access memory outside their sandbox
- **Capability-Based Security**: Tools only get explicitly granted permissions
- **No System Access**: Tools cannot directly access files, network, or OS resources
- **Crash Isolation**: A tool crash doesn't affect other tools or the host system

### ⚡ Performance Advantages

**Near-Native Speed:**
- WASM compiles to efficient machine code
- No interpretation overhead like traditional scripting languages
- Optimized by mature compiler toolchains (LLVM, etc.)

**Fast Startup:**
```
Traditional Process:     ~100-500ms
WASM Component:         ~1-5ms
```

**Memory Efficiency:**
- Linear memory model with precise garbage collection
- No language runtime overhead for compiled languages
- Shared-nothing architecture reduces memory pressure

### 🌍 Universal Portability

**Write Once, Run Anywhere:**
```
Source Code (Rust/Python/Go)
         ↓
    Compile to WASM
         ↓
Run on: macOS | Linux | Windows | Cloud | Browser
```

**Consistent Behavior:**
- Identical execution across all platforms
- No differences in floating-point operations, memory layout, or system calls
- Same security guarantees everywhere

## Real-World Impact

Let's compare FTL's approach with alternatives:

### Traditional Approach: Native Binaries
```bash
# Different binaries needed for each platform
tool-macos-arm64
tool-linux-x86_64  
tool-windows-x86_64
# Security through OS permissions (weak)
# Direct system access (dangerous)
```

### Container Approach: Docker
```bash
# Heavy containers for each tool
docker run --rm tool1:latest  # ~100MB+ per tool
docker run --rm tool2:latest  # Another ~100MB+
# Better isolation but high overhead
# Still platform-specific base images
```

### FTL Approach: WebAssembly
```bash
# Single .wasm file runs everywhere
tool1.wasm          # ~1-5MB, works on all platforms
tool2.wasm          # Strong security, fast startup
# Universal binary format
```

## The Component Model Advantage

Standard WebAssembly is great for single-language applications, but FTL uses the **WebAssembly Component Model** which enables:

### Language Interoperability
```rust
// Rust tool
#[tool]
fn process_data(input: String) -> String { ... }
```

```python
# Python tool  
@tool
def analyze_data(data: str) -> str: ...
```

Both compile to components that can call each other seamlessly.

### Interface Types
```wit
// Shared interface definition
interface math-tools {
  add: func(a: f64, b: f64) -> f64
  multiply: func(numbers: list<f64>) -> f64  
}
```

This interface can be implemented in any language and called from any other language.

### Composition Without Coordination
Tools written by different teams in different languages can work together without:
- Shared dependencies
- Version conflicts  
- Runtime coordination
- Protocol negotiation

## Why Not Alternatives?

### Why Not Native Processes?
- ❌ **Security**: Weak isolation, full system access
- ❌ **Portability**: Platform-specific binaries
- ❌ **Overhead**: Process creation is expensive
- ❌ **Dependencies**: DLL hell, version conflicts

### Why Not Containers?
- ❌ **Size**: Hundreds of MB per tool
- ❌ **Startup**: Slow container initialization
- ❌ **Complexity**: Image management, orchestration
- ❌ **Resource Usage**: High memory and storage overhead

### Why Not JavaScript/V8?
- ❌ **Language Lock-in**: JavaScript only (mostly)
- ❌ **Performance**: Interpretation overhead
- ❌ **Memory Model**: Garbage collection pauses
- ❌ **Standards**: Proprietary runtime APIs

### Why Not Language-Specific Solutions?
- ❌ **Silos**: Python can't easily call Rust, Go can't call Python
- ❌ **FFI Complexity**: Foreign function interfaces are fragile
- ❌ **Deployment**: Multiple runtimes needed
- ❌ **Security**: Shared memory spaces

## FTL's WebAssembly Benefits

By choosing WebAssembly, FTL delivers:

✅ **True Polyglot Programming**: Write each tool in the best language for the job  
✅ **Zero Trust Security**: Every tool runs in a secure sandbox  
✅ **Near-Native Performance**: Fast execution with minimal overhead  
✅ **Universal Deployment**: One binary format for all platforms  
✅ **Minimal Dependencies**: No language runtimes needed in production  
✅ **Future-Proof**: Built on evolving W3C standards  

## Learning More

- **Component Model**: Learn how components communicate in [The Component Model](./component-model.md)
- **Architecture**: See how WASM fits into FTL's overall design in [FTL Architecture](./architecture.md)
- **Implementation**: Understand the development workflow in [Project Lifecycle](./lifecycle.md)

WebAssembly isn't just a technical choice for FTL - it's the foundation that makes true polyglot AI tools possible while maintaining security, performance, and portability.