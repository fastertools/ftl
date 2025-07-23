use ftl_sdk::*;
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn test_tool_metadata_serialization() {
    let metadata = ToolMetadata {
        name: "test-tool".to_string(),
        title: Some("Test Tool".to_string()),
        description: Some("A tool for testing".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            },
            "required": ["input"]
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "result": { "type": "string" }
            }
        })),
        annotations: Some(ToolAnnotations {
            title: None,
            read_only_hint: Some(true),
            destructive_hint: Some(false),
            idempotent_hint: Some(true),
            open_world_hint: None,
        }),
        meta: None,
    };

    let serialized = serde_json::to_value(&metadata);
    assert!(
        serialized.is_ok(),
        "Failed to serialize ToolMetadata to JSON"
    );
    let serialized = serialized.unwrap_or_else(|_| json!({}));

    // Check field names are properly transformed
    assert!(serialized.get("name").is_some());
    assert!(serialized.get("inputSchema").is_some());
    assert!(serialized.get("outputSchema").is_some());

    // Check nested annotations
    let annotations = serialized.get("annotations");
    assert!(
        annotations.is_some(),
        "annotations field should be present in serialized metadata"
    );

    if let Some(annotations) = annotations {
        assert!(annotations.get("readOnlyHint").is_some());
        let read_only_hint = annotations.get("readOnlyHint");
        assert!(
            read_only_hint.is_some(),
            "readOnlyHint should be present in annotations"
        );
        assert_eq!(read_only_hint.unwrap_or(&json!(null)), &json!(true));
    }
}

#[test]
fn test_tool_response_convenience_methods() {
    // Test text response
    let text_response = ToolResponse::text("Hello, world!");
    assert_eq!(text_response.content.len(), 1);

    let first_content = text_response.content.first();
    assert!(
        first_content.is_some(),
        "Expected at least one content item"
    );

    if let Some(ToolContent::Text { text, .. }) = first_content {
        assert_eq!(text, "Hello, world!");
    } else {
        // Force test failure by asserting something that's false
        assert!(
            matches!(first_content, Some(ToolContent::Text { .. })),
            "Expected text content"
        );
    }
    assert!(text_response.is_error.is_none());

    // Test error response
    let error_response = ToolResponse::error("Something went wrong");
    assert_eq!(error_response.is_error, Some(true));

    let first_error_content = error_response.content.first();
    assert!(
        first_error_content.is_some(),
        "Expected at least one content item in error response"
    );

    if let Some(ToolContent::Text { text, .. }) = first_error_content {
        assert_eq!(text, "Something went wrong");
    } else {
        assert!(
            matches!(first_error_content, Some(ToolContent::Text { .. })),
            "Expected text content in error response"
        );
    }

    // Test structured response
    let structured_data = json!({ "result": 42, "status": "success" });
    let structured_response =
        ToolResponse::with_structured("Operation complete", structured_data.clone());
    assert_eq!(
        structured_response.structured_content,
        Some(structured_data)
    );
}

#[test]
fn test_tool_content_serialization() {
    let text_content = ToolContent::Text {
        text: "Sample text".to_string(),
        annotations: Some(ContentAnnotations {
            audience: Some(vec!["developers".to_string()]),
            priority: Some(0.8),
        }),
    };

    let serialized = serde_json::to_value(&text_content);
    assert!(
        serialized.is_ok(),
        "Failed to serialize ToolContent to JSON"
    );
    let serialized = serialized.unwrap_or_else(|_| json!({}));

    let type_field = serialized.get("type");
    assert!(
        type_field.is_some(),
        "type field should be present in serialized content"
    );
    assert_eq!(type_field.unwrap_or(&json!(null)), "text");

    let text_field = serialized.get("text");
    assert!(
        text_field.is_some(),
        "text field should be present in serialized content"
    );
    assert_eq!(text_field.unwrap_or(&json!(null)), "Sample text");

    let annotations = serialized.get("annotations");
    assert!(
        annotations.is_some(),
        "annotations field should be present in serialized content"
    );

    if let Some(annotations) = annotations {
        let audience_field = annotations.get("audience");
        assert!(
            audience_field.is_some(),
            "audience field should be present in annotations"
        );
        assert_eq!(
            audience_field.unwrap_or(&json!(null)),
            &json!(["developers"])
        );
    }
}

#[test]
fn test_image_content() {
    let image = ToolContent::image("base64data", "image/png");
    let serialized = serde_json::to_value(&image);
    assert!(
        serialized.is_ok(),
        "Failed to serialize image ToolContent to JSON"
    );
    let serialized = serialized.unwrap_or_else(|_| json!({}));

    let type_field = serialized.get("type");
    assert!(
        type_field.is_some(),
        "type field should be present in serialized image content"
    );
    assert_eq!(type_field.unwrap_or(&json!(null)), "image");

    let data_field = serialized.get("data");
    assert!(
        data_field.is_some(),
        "data field should be present in serialized image content"
    );
    assert_eq!(data_field.unwrap_or(&json!(null)), "base64data");

    let mime_type_field = serialized.get("mimeType");
    assert!(
        mime_type_field.is_some(),
        "mimeType field should be present in serialized image content"
    );
    assert_eq!(mime_type_field.unwrap_or(&json!(null)), "image/png");
}

#[test]
fn test_resource_content() {
    let resource = ToolContent::Resource {
        resource: ResourceContents {
            uri: "file:///example.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: Some("File contents".to_string()),
            blob: None,
        },
        annotations: None,
    };

    let serialized = serde_json::to_value(&resource);
    assert!(
        serialized.is_ok(),
        "Failed to serialize resource ToolContent to JSON"
    );
    let serialized = serialized.unwrap_or_else(|_| json!({}));

    let type_field = serialized.get("type");
    assert!(
        type_field.is_some(),
        "type field should be present in serialized resource content"
    );
    assert_eq!(type_field.unwrap_or(&json!(null)), "resource");

    let resource_data = serialized.get("resource");
    assert!(
        resource_data.is_some(),
        "resource field should be present in serialized resource content"
    );

    if let Some(resource_data) = resource_data {
        let uri_field = resource_data.get("uri");
        assert!(
            uri_field.is_some(),
            "uri field should be present in resource data"
        );
        assert_eq!(uri_field.unwrap_or(&json!(null)), "file:///example.txt");

        let mime_type_field = resource_data.get("mimeType");
        assert!(
            mime_type_field.is_some(),
            "mimeType field should be present in resource data"
        );
        assert_eq!(mime_type_field.unwrap_or(&json!(null)), "text/plain");
    }
}

#[test]
fn test_optional_fields_are_excluded() {
    let minimal_metadata = ToolMetadata {
        name: "minimal".to_string(),
        title: None,
        description: None,
        input_schema: json!({}),
        output_schema: None,
        annotations: None,
        meta: None,
    };

    let serialized = serde_json::to_value(&minimal_metadata);
    assert!(
        serialized.is_ok(),
        "Failed to serialize minimal ToolMetadata to JSON"
    );
    let serialized = serialized.unwrap_or_else(|_| json!({}));

    // These fields should not be present when None
    assert!(serialized.get("title").is_none());
    assert!(serialized.get("description").is_none());
    assert!(serialized.get("outputSchema").is_none());
    assert!(serialized.get("annotations").is_none());
    assert!(serialized.get("_meta").is_none());
}

#[test]
fn test_round_trip_serialization() {
    let original = ToolResponse {
        content: vec![
            ToolContent::Text {
                text: "First item".to_string(),
                annotations: None,
            },
            ToolContent::Image {
                data: "imagedata".to_string(),
                mime_type: "image/jpeg".to_string(),
                annotations: Some(ContentAnnotations {
                    audience: None,
                    priority: Some(0.5),
                }),
            },
        ],
        structured_content: Some(json!({ "complex": { "nested": "data" } })),
        is_error: Some(false),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&original);
    assert!(
        json.is_ok(),
        "Failed to serialize ToolResponse to JSON string"
    );
    let json = json.unwrap_or_else(|_| String::new());

    // Deserialize back
    let deserialized: Result<ToolResponse, _> = serde_json::from_str(&json);
    assert!(
        deserialized.is_ok(),
        "Failed to deserialize ToolResponse from JSON string"
    );

    if let Ok(deserialized) = deserialized {
        // Compare
        assert_eq!(original.content.len(), deserialized.content.len());
        assert_eq!(original.structured_content, deserialized.structured_content);
        assert_eq!(original.is_error, deserialized.is_error);
    }
}
