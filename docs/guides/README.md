# How-to Guides

These guides provide practical solutions to specific problems you'll encounter when building FTL applications. Each guide is self-contained and focuses on achieving a particular goal.

## Available Guides

### [Making HTTP Requests](./http-requests.md)
**Problem**: Your tool needs to call external APIs or web services  
**Solution**: Configure outbound network access and implement HTTP client code  
**Use cases**: Calling OpenAI API, fetching data from REST APIs, webhook notifications

### [Handling Authentication](./authentication.md)  
**Problem**: Your MCP server needs to authenticate and authorize users  
**Solution**: Configure OAuth 2.0 with JWT tokens using the mcp-authorizer  
**Use cases**: Enterprise deployments, user-specific tool access, API key management

### [Testing Your Tools](./testing.md)
**Problem**: How to write and run tests for your FTL tools  
**Solution**: Language-specific testing strategies and integration test patterns  
**Use cases**: Unit tests, integration tests, CI/CD validation

## Guide Format

Each guide follows the same structure:

1. **Problem Statement**: What specific challenge you're facing
2. **Solution Overview**: High-level approach to solving it  
3. **Step-by-Step Instructions**: Detailed implementation steps
4. **Code Examples**: Working code you can copy and adapt
5. **Troubleshooting**: Common issues and how to solve them
6. **Next Steps**: Related guides and advanced topics

## When to Use Guides

- ✅ **You have a specific goal**: "I need to call an external API"
- ✅ **You want a working solution**: Complete, tested code examples
- ✅ **You're solving a common problem**: Patterns others have needed
- ✅ **You need it to work now**: Practical over theoretical

## When to Use Other Sections

- **New to FTL?** Start with [Getting Started](../getting-started/)
- **Want to understand how it works?** Read [Core Concepts](../core-concepts/)  
- **Need API details?** Check [SDK Reference](../sdk-reference/)
- **Want to contribute?** See [Contributing](../contributing/)

## Requesting New Guides

Missing a guide for your use case? We'd love to hear about it:

1. Check existing [Issues](https://github.com/fastertools/ftl/issues) for similar requests
2. Create a new issue with the `documentation` label
3. Describe your specific problem and use case
4. Include any code examples or approaches you've tried

## Contributing Guides

Found a solution to a common problem? Consider contributing:

1. Follow the standard guide format
2. Include working, tested code examples
3. Test the guide with someone unfamiliar with the solution
4. Submit a pull request with the new guide

The best guides come from real problems you've solved in your own projects!