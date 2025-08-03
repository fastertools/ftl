"""
Validation utilities for FTL SDK.

This module contains validation logic for tool inputs and outputs,
including schema generation and output validation.
"""

import inspect
from typing import Any, Dict, Optional, Union, get_type_hints, get_origin, get_args
import json


class OutputSchemaGenerator:
    """Generate JSON schemas from Python type annotations."""
    
    @staticmethod
    def generate_from_type(python_type: type) -> Optional[Dict[str, Any]]:
        """
        Generate JSON Schema from a Python type annotation.
        
        Args:
            python_type: The Python type to convert
            
        Returns:
            JSON Schema dict or None if schema cannot be generated
        """
        # Handle None/Any types - no schema
        if python_type is None or python_type is type(None) or python_type is Any:
            return None
            
        # Basic type mapping
        type_map = {
            str: {"type": "string"},
            int: {"type": "integer"},
            float: {"type": "number"},
            bool: {"type": "boolean"},
            list: {"type": "array"},
            dict: {"type": "object"}
        }
        
        # Handle primitive types that need wrapping (check this FIRST)
        if python_type in (int, float, str, bool):
            schema = type_map[python_type]
            return {
                "type": "object",
                "properties": {
                    "result": schema
                },
                "required": ["result"],
                "x-ftl-wrapped": True
            }
        
        # Check for direct mapping (non-primitives)
        if python_type in type_map:
            return type_map[python_type]
            
        # Handle Union types (including Optional)
        origin = get_origin(python_type)
        if origin is Union:
            args = get_args(python_type)
            # Check if it's Optional (Union with None)
            if type(None) in args:
                non_none_types = [t for t in args if t is not type(None)]
                if len(non_none_types) == 1:
                    # Optional type - generate schema for inner type
                    inner_schema = OutputSchemaGenerator.generate_from_type(non_none_types[0])
                    if inner_schema:
                        # For primitives, wrap in object schema
                        if inner_schema.get("type") != "object":
                            return {
                                "type": "object",
                                "properties": {
                                    "result": inner_schema
                                },
                                "required": ["result"],
                                "x-ftl-wrapped": True
                            }
                        return inner_schema
            return None
            
        # Handle List types
        if origin is list:
            args = get_args(python_type)
            if args:
                item_schema = OutputSchemaGenerator.generate_from_type(args[0])
                if item_schema:
                    # List types need to be wrapped in object
                    return {
                        "type": "object",
                        "properties": {
                            "result": {
                                "type": "array",
                                "items": item_schema
                            }
                        },
                        "required": ["result"],
                        "x-ftl-wrapped": True
                    }
            return None
            
        # Handle Dict types - already objects, no wrapping needed
        if origin is dict:
            return {"type": "object"}
            
        # For other types (classes, etc.), assume object
        return {"type": "object"}


class OutputValidator:
    """Validate tool outputs against their schemas."""
    
    @staticmethod
    def validate_output(output: Any, schema: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """
        Validate and transform output according to schema.
        
        Args:
            output: The output value to validate
            schema: The JSON schema to validate against
            
        Returns:
            Validated and potentially transformed output
            
        Raises:
            ValueError: If output doesn't match schema
        """
        if schema is None:
            # No schema - no validation needed
            return output if isinstance(output, dict) else {"result": output}
            
        # Check if schema indicates wrapped primitive
        if schema.get("x-ftl-wrapped"):
            # Output should be wrapped in result key
            if not isinstance(output, dict):
                # Wrap primitive value
                return {"result": output}
            elif "result" not in output:
                # Wrap dict in result key
                return {"result": output}
                
        # For object schemas without wrapping, ensure output is dict
        if schema.get("type") == "object" and not isinstance(output, dict):
            # Try to convert to dict
            if hasattr(output, "__dict__"):
                return vars(output)
            else:
                # Can't convert to dict - wrap it
                return {"value": output}
                
        return output
    
    @staticmethod
    def is_valid_structured_output(output: Any) -> bool:
        """
        Check if output is valid for structured content.
        
        Structured content must be JSON-serializable dict.
        
        Args:
            output: The output to check
            
        Returns:
            True if valid for structured content
        """
        if not isinstance(output, dict):
            return False
            
        try:
            # Try to serialize to JSON
            json.dumps(output)
            return True
        except (TypeError, ValueError):
            return False