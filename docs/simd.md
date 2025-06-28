# SIMD

This document provides practical examples of how to use SIMD (Single Instruction, Multiple Data) instructions in FTL Core tools for high-performance vector operations.

## Overview

FTL Core has [SIMD128 support](https://github.com/WebAssembly/spec/blob/main/proposals/simd/SIMD.md) enabled by default. This allows you to:

1. **Automatic vectorization**: Let the compiler optimize your loops
2. **Explicit SIMD**: Use WebAssembly SIMD intrinsics directly
3. **Performance gains**: Up to 4x speedup for compatible operations

## Automatic Vectorization

The Rust compiler with SIMD enabled will automatically optimize many common patterns:

```rust
// This function will be automatically vectorized
pub fn add_arrays(a: &[f32], b: &[f32]) -> Vec<f32> {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x + y)
        .collect()
}

// Sum reduction will use SIMD
pub fn sum_array(data: &[f32]) -> f32 {
    data.iter().sum()
}

// Element-wise operations get vectorized
pub fn scale_array(data: &[f32], scale: f32) -> Vec<f32> {
    data.iter().map(|x| x * scale).collect()
}
```

## Explicit SIMD Intrinsics

For maximum control, use WebAssembly SIMD intrinsics:

```rust
use std::arch::wasm32::*;

pub fn simd_add_f32_arrays(a: &[f32], b: &[f32]) -> Vec<f32> {
    assert_eq!(a.len(), b.len());
    let mut result = Vec::with_capacity(a.len());

    // Process 4 elements at a time
    let chunks = a.len() / 4;
    for i in 0..chunks {
        let base = i * 4;

        // Load 4 f32 values into SIMD vectors
        let va = v128_load(a.as_ptr().add(base) as *const v128);
        let vb = v128_load(b.as_ptr().add(base) as *const v128);

        // Perform vectorized addition
        let sum = f32x4_add(va, vb);

        // Store result
        let mut temp = [0.0f32; 4];
        v128_store(temp.as_mut_ptr() as *mut v128, sum);
        result.extend_from_slice(&temp);
    }

    // Handle remaining elements
    for i in (chunks * 4)..a.len() {
        result.push(a[i] + b[i]);
    }

    result
}

pub fn simd_dot_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    let mut sum_vec = f32x4_splat(0.0);
    let chunks = a.len() / 4;

    for i in 0..chunks {
        let base = i * 4;
        let va = v128_load(a.as_ptr().add(base) as *const v128);
        let vb = v128_load(b.as_ptr().add(base) as *const v128);
        let product = f32x4_mul(va, vb);
        sum_vec = f32x4_add(sum_vec, product);
    }

    // Extract and sum the 4 accumulated values
    let mut temp = [0.0f32; 4];
    v128_store(temp.as_mut_ptr() as *mut v128, sum_vec);
    let simd_sum = temp.iter().sum::<f32>();

    // Add remaining elements
    let remainder_sum: f32 = a[(chunks * 4)..]
        .iter()
        .zip(&b[(chunks * 4)..])
        .map(|(x, y)| x * y)
        .sum();

    simd_sum + remainder_sum
}
```

## Cryptographic Example

SIMD is particularly useful for cryptographic operations:

```rust
use std::arch::wasm32::*;

pub fn simd_xor_cipher(data: &[u8], key: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());
    let key_len = key.len();

    // Process 16 bytes at a time
    let chunks = data.len() / 16;
    for i in 0..chunks {
        let base = i * 16;

        // Create repeating key pattern
        let mut key_block = [0u8; 16];
        for j in 0..16 {
            key_block[j] = key[(base + j) % key_len];
        }

        // Load data and key as SIMD vectors
        let data_vec = v128_load(data.as_ptr().add(base) as *const v128);
        let key_vec = v128_load(key_block.as_ptr() as *const v128);

        // Perform vectorized XOR
        let xor_result = v128_xor(data_vec, key_vec);

        // Store result
        let mut temp = [0u8; 16];
        v128_store(temp.as_mut_ptr() as *mut v128, xor_result);
        result.extend_from_slice(&temp);
    }

    // Handle remaining bytes
    for i in (chunks * 16)..data.len() {
        result.push(data[i] ^ key[i % key_len]);
    }

    result
}
```

## Integer Operations

SIMD works well with integer operations too:

```rust
use std::arch::wasm32::*;

pub fn simd_sum_i32(data: &[i32]) -> i32 {
    let mut sum_vec = i32x4_splat(0);
    let chunks = data.len() / 4;

    for i in 0..chunks {
        let base = i * 4;
        let vec = v128_load(data.as_ptr().add(base) as *const v128);
        sum_vec = i32x4_add(sum_vec, vec);
    }

    // Extract and sum the 4 accumulated values
    let mut temp = [0i32; 4];
    v128_store(temp.as_mut_ptr() as *mut v128, sum_vec);
    let simd_sum = temp.iter().sum::<i32>();

    // Add remaining elements
    let remainder_sum: i32 = data[(chunks * 4)..].iter().sum();

    simd_sum + remainder_sum
}

pub fn simd_find_max_i32(data: &[i32]) -> Option<i32> {
    if data.is_empty() {
        return None;
    }

    let mut max_vec = i32x4_splat(i32::MIN);
    let chunks = data.len() / 4;

    for i in 0..chunks {
        let base = i * 4;
        let vec = v128_load(data.as_ptr().add(base) as *const v128);
        max_vec = i32x4_max(max_vec, vec);
    }

    // Extract the 4 accumulated values
    let mut temp = [0i32; 4];
    v128_store(temp.as_mut_ptr() as *mut v128, max_vec);
    let simd_max = temp.iter().max().copied().unwrap_or(i32::MIN);

    // Check remaining elements
    let remainder_max = data[(chunks * 4)..].iter().max().copied().unwrap_or(i32::MIN);

    Some(simd_max.max(remainder_max))
}
```

## Integration with MCP Tools

Here's how you might integrate SIMD into an MCP tool:

```rust
use ftl_mcp_server::prelude::*;
use serde_json::{json, Value};
use std::arch::wasm32::*;

#[derive(Clone)]
pub struct VectorMathTool;

impl Tool for VectorMathTool {
    fn name(&self) -> &'static str {
        "vector-math"
    }

    fn description(&self) -> &'static str {
        "High-performance vector mathematical operations using SIMD"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "multiply", "dot_product", "sum"]
                },
                "vector_a": {
                    "type": "array",
                    "items": {"type": "number"}
                },
                "vector_b": {
                    "type": "array",
                    "items": {"type": "number"}
                }
            },
            "required": ["operation", "vector_a"]
        })
    }

    fn call(&self, args: &Value) -> Result<ToolResult, ToolError> {
        let operation = args["operation"].as_str()
            .ok_or_else(|| ToolError::InvalidInput("Missing operation".to_string()))?;

        let vector_a: Vec<f32> = args["vector_a"].as_array()
            .ok_or_else(|| ToolError::InvalidInput("Missing vector_a".to_string()))?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        let result = match operation {
            "sum" => {
                let sum = simd_sum_f32(&vector_a);
                json!({"result": sum})
            }
            "add" => {
                let vector_b: Vec<f32> = args["vector_b"].as_array()
                    .ok_or_else(|| ToolError::InvalidInput("Missing vector_b for add operation".to_string()))?
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();

                if vector_a.len() != vector_b.len() {
                    return Err(ToolError::InvalidInput("Vectors must have same length".to_string()));
                }

                let result = simd_add_f32_arrays(&vector_a, &vector_b);
                json!({"result": result})
            }
            "dot_product" => {
                let vector_b: Vec<f32> = args["vector_b"].as_array()
                    .ok_or_else(|| ToolError::InvalidInput("Missing vector_b for dot_product".to_string()))?
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();

                if vector_a.len() != vector_b.len() {
                    return Err(ToolError::InvalidInput("Vectors must have same length".to_string()));
                }

                let result = simd_dot_product(&vector_a, &vector_b);
                json!({"result": result})
            }
            _ => return Err(ToolError::InvalidInput(format!("Unknown operation: {}", operation)))
        };

        Ok(ToolResult::content(vec![Content::text(result.to_string())]))
    }
}

// Helper function for SIMD sum
fn simd_sum_f32(data: &[f32]) -> f32 {
    let mut sum_vec = f32x4_splat(0.0);
    let chunks = data.len() / 4;

    for i in 0..chunks {
        let base = i * 4;
        let vec = v128_load(data.as_ptr().add(base) as *const v128);
        sum_vec = f32x4_add(sum_vec, vec);
    }

    let mut temp = [0.0f32; 4];
    v128_store(temp.as_mut_ptr() as *mut v128, sum_vec);
    let simd_sum = temp.iter().sum::<f32>();

    let remainder_sum: f32 = data[(chunks * 4)..].iter().sum();

    simd_sum + remainder_sum
}

ftl_mcp_server!(VectorMathTool);
```

## Performance Tips

1. **Alignment**: While not required, aligned data can improve performance
2. **Chunk size**: Process data in multiples of SIMD width (4 for f32, 16 for u8)
3. **Hot paths**: Focus SIMD optimization on performance-critical loops
4. **Measure**: Always benchmark - sometimes automatic vectorization is sufficient
5. **Fallback**: Handle remainder elements that don't fit in SIMD chunks

## Testing SIMD Code

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_add_arrays() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let result = simd_add_f32_arrays(&a, &b);
        let expected = vec![2.0, 3.0, 4.0, 5.0, 6.0];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_simd_dot_product() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![2.0, 2.0, 2.0, 2.0];
        let result = simd_dot_product(&a, &b);
        let expected = 20.0; // 1*2 + 2*2 + 3*2 + 4*2

        assert!((result - expected).abs() < f32::EPSILON);
    }
}
```

## When to Use SIMD

**Good candidates:**
- Mathematical operations on arrays/vectors
- Cryptographic functions (hashing, encryption)
- Image/signal processing
- Data transformations and filtering
- Text processing (character operations)

**Not suitable for:**
- Single value operations
- Complex branching logic
- Operations requiring scalar reduction
- Non-uniform data access patterns

## Conclusion

SIMD support in FTL Core enables significant performance improvements for vector operations. Start with automatic vectorization for most cases, and use explicit SIMD intrinsics when you need maximum control and performance.
