# Introduction

Welcome to the FTL documentation! This document provides a comprehensive overview of the FTL project, its goals, and its core concepts.

## What is FTL?

FTL is a framework for building and deploying high-performance, low-latency tools for AI agents. It is designed to solve the "Action Latency Bottleneck," which is the problem of slow tool execution that can limit the performance of AI agents in real-time applications.

FTL provides a complete developer experience for the entire lifecycle of creating, testing, and deploying WebAssembly-based tools. It is built on a foundation of Rust, WebAssembly, and the Model Context Protocol (MCP), and it is designed to be fast, secure, and portable.

## Core Concepts

### Tools

A **tool** is a self-contained piece of code that performs a specific task. Tools are implemented in Rust by implementing the `ftl_sdk_rs::Tool` trait. They are compiled to WebAssembly and can be deployed to any Wasm-compliant runtime.

### Toolkits

A **toolkit** is a collection of tools that are deployed together as a single unit. This allows you to create more complex agent capabilities by composing multiple tools.

### FTL Core

**FTL Core** is an open-source library of composable, low-level utilities for performance-sensitive AI agents. It provides the building blocks for creating tools, as well as a standard library of pre-built tools for common tasks.

### FTL Edge

**FTL Edge** is a managed platform for deploying and serving tools. It provides a global network of edge servers that can execute tools with sub-millisecond compute overhead.

## Why FTL?

FTL is designed to be the best way to build and deploy high-performance tools for AI agents. It offers a number of advantages over other tool-building frameworks:

- **Performance:** FTL tools are written in Rust and compiled to WebAssembly, which provides near-native performance.
- **Security:** FTL tools are sandboxed by default using the WebAssembly component model.
- **Portability:** FTL tools can be deployed to any Wasm-compliant runtime.
- **Developer Experience:** The `ftl` CLI provides a seamless developer experience for creating, testing, and deploying tools.
- **Open Core:** FTL is an open-core project, which means that the core technology is open source and available to everyone.
