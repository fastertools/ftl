use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, parse_macro_input};

/// Define multiple tools in a single component.
///
/// This macro allows you to define all your tools in one place, automatically
/// generating the HTTP handler and metadata for each tool.
///
/// # Example
/// ```ignore
/// ftl_tools! {
///     /// Echo back the input message
///     fn echo(input: EchoInput) -> ToolResponse {
///         ToolResponse::text(format!("Echo: {}", input.message))
///     }
///
///     /// Reverse the input text
///     fn reverse(input: ReverseInput) -> ToolResponse {
///         ToolResponse::text(input.text.chars().rev().collect::<String>())
///     }
/// }
/// ```
#[proc_macro]
pub fn ftl_tools(input: TokenStream) -> TokenStream {
    let tools = parse_macro_input!(input as ToolsDefinition);

    // Collect all tool functions
    let tool_fns: Vec<_> = tools.functions.iter().collect();

    // Generate metadata for each tool
    let metadata_items: Vec<_> = tool_fns.iter().map(|func| {
        let name = &func.sig.ident;
        let name_str = name.to_string();

        // Extract doc comment
        let description = extract_doc_comment(&func.attrs)
            .map(|d| quote!(Some(#d.to_string())))
            .unwrap_or(quote!(None));

        // Get input type
        let input_type = match func.sig.inputs.first() {
            Some(FnArg::Typed(pat_type)) => &pat_type.ty,
            _ => panic!("Tool function must have exactly one typed argument"),
        };

        quote! {
            ::ftl_sdk::ToolMetadata {
                name: #name_str.to_string(),
                title: Some(generate_title(#name_str)),
                description: #description,
                input_schema: ::serde_json::to_value(::schemars::schema_for!(#input_type)).unwrap(),
                output_schema: None,
                annotations: None,
                meta: None,
            }
        }
    }).collect();

    // Generate routing cases for POST requests
    let routing_cases: Vec<_> = tool_fns.iter().map(|func| {
        let name = &func.sig.ident;
        let name_str = name.to_string();
        let is_async = func.sig.asyncness.is_some();

        // Get input type
        let input_type = match func.sig.inputs.first() {
            Some(FnArg::Typed(pat_type)) => &pat_type.ty,
            _ => panic!("Tool function must have exactly one typed argument"),
        };

        let fn_call = if is_async {
            quote!(#name(input).await)
        } else {
            quote!(#name(input))
        };

        quote! {
            #name_str => {
                match ::serde_json::from_slice::<#input_type>(body) {
                    Ok(input) => {
                        let response = #fn_call;
                        match ::serde_json::to_vec(&response) {
                            Ok(body) => Response::builder()
                                .status(200)
                                .header("Content-Type", "application/json")
                                .body(body)
                                .build(),
                            Err(e) => {
                                let error_response = ::ftl_sdk::ToolResponse::error(
                                    format!("Failed to serialize response: {}", e)
                                );
                                Response::builder()
                                    .status(500)
                                    .header("Content-Type", "application/json")
                                    .body(::serde_json::to_vec(&error_response).unwrap_or_default())
                                    .build()
                            }
                        }
                    }
                    Err(e) => {
                        let error_response = ::ftl_sdk::ToolResponse::error(
                            format!("Invalid request body: {}", e)
                        );
                        Response::builder()
                            .status(400)
                            .header("Content-Type", "application/json")
                            .body(::serde_json::to_vec(&error_response).unwrap_or_default())
                            .build()
                    }
                }
            }
        }
    }).collect();

    let output = quote! {
        // Define all tool functions
        #(#tool_fns)*

        // Generate the HTTP component handler
        #[::spin_sdk::http_component]
        async fn handle_tool_component(req: ::spin_sdk::http::Request) -> ::spin_sdk::http::Response {
            use ::spin_sdk::http::{Method, Response};

            // Helper function to generate title from name
            fn generate_title(name: &str) -> String {
                name.split('_')
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }

            let path = req.path();

            match req.method() {
                &Method::Get if path == "/" => {
                    // Return metadata for all tools
                    let tools = vec![
                        #(#metadata_items),*
                    ];

                    match ::serde_json::to_vec(&tools) {
                        Ok(body) => Response::builder()
                            .status(200)
                            .header("Content-Type", "application/json")
                            .body(body)
                            .build(),
                        Err(e) => Response::builder()
                            .status(500)
                            .body(format!("Failed to serialize metadata: {}", e))
                            .build()
                    }
                }
                &Method::Post => {
                    // Get the tool name from the path
                    let tool_name = path.trim_start_matches('/');
                    let body = req.body();

                    match tool_name {
                        #(#routing_cases)*
                        _ => {
                            let error_response = ::ftl_sdk::ToolResponse::error(
                                format!("Tool '{}' not found", tool_name)
                            );
                            Response::builder()
                                .status(404)
                                .header("Content-Type", "application/json")
                                .body(::serde_json::to_vec(&error_response).unwrap_or_default())
                                .build()
                        }
                    }
                }
                _ => Response::builder()
                    .status(405)
                    .header("Allow", "GET, POST")
                    .body("Method not allowed")
                    .build()
            }
        }
    };

    output.into()
}

// Parse multiple function definitions
struct ToolsDefinition {
    functions: Vec<ItemFn>,
}

impl syn::parse::Parse for ToolsDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut functions = Vec::new();

        while !input.is_empty() {
            functions.push(input.parse::<ItemFn>()?);
        }

        if functions.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "At least one tool function must be defined",
            ));
        }

        Ok(ToolsDefinition { functions })
    }
}

// Extract the first line of doc comments from attributes
fn extract_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(lit) = &nv.value {
                        if let syn::Lit::Str(s) = &lit.lit {
                            let doc = s.value();
                            // Trim leading space that rustdoc adds
                            return Some(doc.trim_start_matches(' ').to_string());
                        }
                    }
                }
            }
            None
        })
        .next()
}
