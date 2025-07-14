# Performance Optimization Guide

This guide covers performance optimization techniques for FTL applications, focusing on WebAssembly optimization, runtime performance, and efficient resource usage.

## Performance Overview

FTL application performance is influenced by:
- WebAssembly module size and optimization
- Component initialization time
- Request processing efficiency
- Memory usage patterns
- Network latency (for external calls)

## WebAssembly Optimization

### Build Optimization Flags

Always build with release mode for production:

```bash
# Development build (fast compile, slow runtime)
ftl build

# Production build (slow compile, fast runtime)
ftl build --release
```

### Rust Optimization

Configure `Cargo.toml` for optimal performance:

```toml
[profile.release]
opt-level = "z"          # Optimize for size
lto = true               # Link-time optimization
codegen-units = 1        # Single codegen unit
strip = true             # Strip symbols
panic = "abort"          # Smaller panic handler

[profile.release.package."*"]
opt-level = "z"          # Optimize all dependencies
```

Size-optimized build configuration:

```toml
# Alternative: Optimize for speed
[profile.release]
opt-level = 3            # Maximum speed optimization
lto = "fat"              # Full LTO
codegen-units = 1        # Better optimization
```

### TypeScript Optimization

Configure optimal build settings:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ES2022",
    "lib": ["ES2022"],
    "removeComments": true,
    "sourceMap": false,
    "declaration": false,
    "incremental": false
  }
}
```

Use esbuild for bundling:

```javascript
// build.js
const esbuild = require('esbuild');

esbuild.build({
  entryPoints: ['src/index.ts'],
  bundle: true,
  minify: true,
  sourcemap: false,
  target: 'es2022',
  format: 'esm',
  outfile: 'dist/index.js',
  external: ['node:*'],
  treeShaking: true,
  drop: ['console', 'debugger']
});
```

## Memory Optimization

### Efficient Memory Usage

Monitor and limit memory allocation:

```rust
// Rust: Pre-allocate collections
let mut results = Vec::with_capacity(expected_size);

// Avoid unnecessary clones
fn process(data: &str) -> Result<String> {
    // Process without cloning unless necessary
    if needs_modification(data) {
        let mut owned = data.to_string();
        modify(&mut owned);
        Ok(owned)
    } else {
        Ok(data.to_string())
    }
}
```

TypeScript memory considerations:

```typescript
// Reuse objects instead of creating new ones
const objectPool: MyObject[] = [];

function getObject(): MyObject {
  return objectPool.pop() || new MyObject();
}

function releaseObject(obj: MyObject) {
  obj.reset();
  objectPool.push(obj);
}

// Use streaming for large data
async function* processLargeData(data: AsyncIterable<Chunk>) {
  for await (const chunk of data) {
    yield processChunk(chunk);
  }
}
```

### WebAssembly Memory Configuration

Configure memory limits in `spin.toml`:

```toml
[component.my-tool]
# Memory pages (64KB each)
memory = { initial = 10, maximum = 100 }  # 640KB - 6.4MB
```

## Request Processing Optimization

### Async Processing

Use async operations efficiently:

```rust
// Good: Concurrent processing
use futures::future::join_all;

async fn process_multiple(items: Vec<Item>) -> Vec<Result<Output>> {
    let futures = items.into_iter()
        .map(|item| async move { process_item(item).await });
    
    join_all(futures).await
}

// Bad: Sequential processing
async fn process_multiple_slow(items: Vec<Item>) -> Vec<Result<Output>> {
    let mut results = vec![];
    for item in items {
        results.push(process_item(item).await);
    }
    results
}
```

### Caching Strategies

Implement efficient caching:

```typescript
// LRU Cache implementation
class LRUCache<K, V> {
  private cache = new Map<K, V>();
  private maxSize: number;

  constructor(maxSize: number) {
    this.maxSize = maxSize;
  }

  get(key: K): V | undefined {
    const value = this.cache.get(key);
    if (value !== undefined) {
      // Move to end (most recently used)
      this.cache.delete(key);
      this.cache.set(key, value);
    }
    return value;
  }

  set(key: K, value: V): void {
    if (this.cache.size >= this.maxSize) {
      // Remove least recently used
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
    this.cache.set(key, value);
  }
}

// Use the cache
const cache = new LRUCache<string, ProcessedData>(100);

export async function handler(input: Input) {
  const cacheKey = generateKey(input);
  const cached = cache.get(cacheKey);
  
  if (cached) {
    return ToolResponse.cached(cached);
  }
  
  const result = await expensiveOperation(input);
  cache.set(cacheKey, result);
  return ToolResponse.fresh(result);
}
```

## Network Optimization

### Connection Pooling

Reuse HTTP connections:

```typescript
// Create a reusable client
const httpClient = new HttpClient({
  keepAlive: true,
  maxSockets: 10,
  timeout: 5000
});

// Bad: Creating new connection each time
async function fetchData(url: string) {
  return fetch(url);
}

// Good: Reusing connections
async function fetchDataOptimized(url: string) {
  return httpClient.get(url);
}
```

### Batch Operations

Reduce network calls through batching:

```typescript
class BatchProcessor<T, R> {
  private batch: T[] = [];
  private timer: NodeJS.Timeout | null = null;
  
  constructor(
    private processBatch: (items: T[]) => Promise<R[]>,
    private maxBatchSize = 100,
    private maxWaitTime = 100
  ) {}
  
  async add(item: T): Promise<R> {
    return new Promise((resolve, reject) => {
      this.batch.push(item);
      
      if (this.batch.length >= this.maxBatchSize) {
        this.flush();
      } else if (!this.timer) {
        this.timer = setTimeout(() => this.flush(), this.maxWaitTime);
      }
    });
  }
  
  private async flush() {
    const items = this.batch;
    this.batch = [];
    this.timer = null;
    
    try {
      const results = await this.processBatch(items);
      // Resolve individual promises
    } catch (error) {
      // Reject individual promises
    }
  }
}
```

## Component Optimization

### Lazy Loading

Load components only when needed:

```typescript
// Lazy load heavy dependencies
let heavyLibrary: HeavyLibrary | null = null;

async function getHeavyLibrary(): Promise<HeavyLibrary> {
  if (!heavyLibrary) {
    const module = await import('./heavy-library');
    heavyLibrary = new module.HeavyLibrary();
  }
  return heavyLibrary;
}

// Use only when needed
export async function handler(input: Input) {
  if (input.requiresHeavyProcessing) {
    const lib = await getHeavyLibrary();
    return lib.process(input);
  }
  return lightweightProcess(input);
}
```

### Code Splitting

Split code into smaller chunks:

```javascript
// webpack.config.js
module.exports = {
  optimization: {
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          priority: 10
        }
      }
    }
  }
};
```

## Benchmarking and Profiling

### Performance Measurement

Add performance monitoring to your code:

```typescript
class PerformanceMonitor {
  private metrics = new Map<string, number[]>();
  
  measure<T>(name: string, fn: () => T): T {
    const start = performance.now();
    try {
      return fn();
    } finally {
      const duration = performance.now() - start;
      const metrics = this.metrics.get(name) || [];
      metrics.push(duration);
      this.metrics.set(name, metrics);
    }
  }
  
  async measureAsync<T>(name: string, fn: () => Promise<T>): Promise<T> {
    const start = performance.now();
    try {
      return await fn();
    } finally {
      const duration = performance.now() - start;
      const metrics = this.metrics.get(name) || [];
      metrics.push(duration);
      this.metrics.set(name, metrics);
    }
  }
  
  getStats(name: string) {
    const metrics = this.metrics.get(name) || [];
    if (metrics.length === 0) return null;
    
    const sorted = [...metrics].sort((a, b) => a - b);
    return {
      count: metrics.length,
      min: sorted[0],
      max: sorted[sorted.length - 1],
      avg: metrics.reduce((a, b) => a + b, 0) / metrics.length,
      p50: sorted[Math.floor(sorted.length * 0.5)],
      p95: sorted[Math.floor(sorted.length * 0.95)],
      p99: sorted[Math.floor(sorted.length * 0.99)]
    };
  }
}
```

### Load Testing

Use artillery for load testing:

```yaml
# load-test.yml
config:
  target: "http://localhost:3000"
  phases:
    - duration: 60
      arrivalRate: 10
      name: "Warm up"
    - duration: 300
      arrivalRate: 50
      name: "Sustained load"
    - duration: 60
      arrivalRate: 100
      name: "Peak load"

scenarios:
  - name: "Tool Execution"
    flow:
      - post:
          url: "/mcp"
          json:
            jsonrpc: "2.0"
            method: "tools/call"
            params:
              name: "my-tool"
              arguments:
                message: "Test message"
            id: 1
```

Run the test:

```bash
artillery run load-test.yml --output results.json
artillery report results.json
```

## Optimization Checklist

### Before Optimization

- [ ] Measure current performance baseline
- [ ] Identify bottlenecks with profiling
- [ ] Set performance targets
- [ ] Prioritize optimizations by impact

### Code Optimization

- [ ] Use release builds
- [ ] Enable compiler optimizations
- [ ] Remove debug code and logging
- [ ] Minimize dependencies
- [ ] Use efficient algorithms
- [ ] Implement caching where appropriate

### Memory Optimization

- [ ] Profile memory usage
- [ ] Fix memory leaks
- [ ] Use object pooling
- [ ] Limit concurrent operations
- [ ] Configure appropriate memory limits

### Network Optimization

- [ ] Enable connection pooling
- [ ] Implement request batching
- [ ] Use compression
- [ ] Minimize payload sizes
- [ ] Cache external API responses

### Monitoring

- [ ] Add performance metrics
- [ ] Set up alerting for degradation
- [ ] Regular performance reviews
- [ ] A/B test optimizations

## Common Performance Issues

### Large Bundle Sizes

**Problem**: WebAssembly modules are too large

**Solutions**:
1. Use `wasm-opt` for additional optimization:
   ```bash
   wasm-opt -Oz input.wasm -o output.wasm
   ```

2. Remove unused code:
   ```toml
   [package.metadata.component]
   features = ["minimal"]
   ```

3. Use dynamic imports for optional features

### Slow Cold Starts

**Problem**: First request takes too long

**Solutions**:
1. Pre-warm components
2. Minimize initialization work
3. Lazy load heavy dependencies
4. Use smaller base images

### Memory Leaks

**Problem**: Memory usage grows over time

**Solutions**:
1. Profile with memory tools
2. Clear caches periodically
3. Use weak references
4. Implement proper cleanup

## Advanced Techniques

### SIMD Optimization

Enable SIMD for compute-intensive operations:

```rust
#[cfg(target_feature = "simd128")]
use core::arch::wasm32::*;

#[cfg(target_feature = "simd128")]
fn process_data_simd(data: &[f32]) -> Vec<f32> {
    // SIMD implementation
}

#[cfg(not(target_feature = "simd128"))]
fn process_data_simd(data: &[f32]) -> Vec<f32> {
    // Fallback implementation
}
```

### Web Workers

Offload CPU-intensive work:

```typescript
// worker.ts
self.addEventListener('message', async (e) => {
  const result = await heavyComputation(e.data);
  self.postMessage(result);
});

// main.ts
const worker = new Worker('worker.js');
worker.postMessage(data);
worker.addEventListener('message', (e) => {
  console.log('Result:', e.data);
});
```

## Performance Budget

Set and enforce performance budgets:

```json
{
  "budgets": {
    "bundleSize": "500KB",
    "memoryUsage": "50MB",
    "coldStart": "500ms",
    "p95Latency": "200ms",
    "errorRate": "0.1%"
  }
}
```

Monitor budget compliance:

```bash
#!/bin/bash
# check-performance.sh

BUNDLE_SIZE=$(stat -f%z dist/*.wasm | awk '{s+=$1} END {print s}')
if [ $BUNDLE_SIZE -gt 512000 ]; then
  echo "ERROR: Bundle size exceeds budget: ${BUNDLE_SIZE} bytes"
  exit 1
fi
```

## Resources

- [WebAssembly Optimization Tips](https://webassembly.org/docs/portability/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [V8 JavaScript Performance](https://v8.dev/docs/performance)
- [Spin Performance Guide](https://developer.fermyon.com/spin/performance)