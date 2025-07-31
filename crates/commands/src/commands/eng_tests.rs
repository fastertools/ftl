//! Tests for engine commands

#[cfg(test)]
mod tests {
    #![allow(clippy::module_inception)]
    use std::sync::Arc;

    use anyhow::Result;
    use async_trait::async_trait;
    use uuid::Uuid;

    use crate::commands::r#eng::{self as eng_cmd, EngDependencies, OutputFormat};
    use ftl_runtime::api_client::types::{self};
    use ftl_runtime::deps::{
        FtlApiClient, MessageStyle, MultiProgressManager, ProgressIndicator, UserInterface,
    };

    // Mock implementations

    #[allow(clippy::struct_field_names)]
    struct MockApiClient {
        list_response: Result<types::ListAppsResponse>,
        get_response: Result<types::App>,
        delete_response: Result<types::DeleteAppResponse>,
    }

    #[async_trait]
    impl FtlApiClient for MockApiClient {
        async fn create_app(
            &self,
            _request: &types::CreateAppRequest,
        ) -> Result<types::CreateAppResponse> {
            unimplemented!()
        }

        async fn list_apps(
            &self,
            _limit: Option<std::num::NonZeroU64>,
            _next_token: Option<&str>,
            _name: Option<&str>,
        ) -> Result<types::ListAppsResponse> {
            match &self.list_response {
                Ok(resp) => Ok(resp.clone()),
                Err(e) => Err(anyhow::anyhow!(e.to_string())),
            }
        }

        async fn get_app(&self, _app_id: &str) -> Result<types::App> {
            match &self.get_response {
                Ok(resp) => Ok(resp.clone()),
                Err(e) => Err(anyhow::anyhow!(e.to_string())),
            }
        }

        async fn delete_app(&self, _app_id: &str) -> Result<types::DeleteAppResponse> {
            match &self.delete_response {
                Ok(resp) => Ok(resp.clone()),
                Err(e) => Err(anyhow::anyhow!(e.to_string())),
            }
        }

        async fn create_deployment(
            &self,
            _app_id: &str,
            _request: &types::CreateDeploymentRequest,
        ) -> Result<types::CreateDeploymentResponse> {
            unimplemented!()
        }

        async fn update_components(
            &self,
            _app_id: &str,
            _request: &types::UpdateComponentsRequest,
        ) -> Result<types::UpdateComponentsResponse> {
            unimplemented!()
        }

        async fn list_app_components(
            &self,
            _app_id: &str,
        ) -> Result<types::ListComponentsResponse> {
            unimplemented!()
        }

        async fn create_ecr_token(&self) -> Result<types::CreateEcrTokenResponse> {
            unimplemented!()
        }

        async fn update_auth_config(
            &self,
            _app_id: &str,
            _request: &types::UpdateAuthConfigRequest,
        ) -> Result<types::AuthConfigResponse> {
            unimplemented!()
        }
    }

    struct MockUI {
        messages: std::sync::Mutex<Vec<String>>,
        styled_messages: std::sync::Mutex<Vec<(String, MessageStyle)>>,
        prompt_responses: std::sync::Mutex<Vec<String>>,
        is_interactive: bool,
    }

    impl MockUI {
        fn new() -> Self {
            Self {
                messages: std::sync::Mutex::new(Vec::new()),
                styled_messages: std::sync::Mutex::new(Vec::new()),
                prompt_responses: std::sync::Mutex::new(Vec::new()),
                is_interactive: true,
            }
        }

        fn with_prompt_response(self, response: &str) -> Self {
            self.prompt_responses
                .lock()
                .unwrap()
                .push(response.to_string());
            self
        }

        fn get_messages(&self) -> Vec<String> {
            self.messages.lock().unwrap().clone()
        }

        fn get_styled_messages(&self) -> Vec<(String, MessageStyle)> {
            self.styled_messages.lock().unwrap().clone()
        }
    }

    impl UserInterface for MockUI {
        fn create_spinner(&self) -> Box<dyn ProgressIndicator> {
            Box::new(MockProgress)
        }

        fn create_multi_progress(&self) -> Box<dyn MultiProgressManager> {
            Box::new(MockMultiProgress)
        }

        fn print(&self, message: &str) {
            self.messages.lock().unwrap().push(message.to_string());
        }

        fn print_styled(&self, message: &str, style: MessageStyle) {
            self.styled_messages
                .lock()
                .unwrap()
                .push((message.to_string(), style));
        }

        fn is_interactive(&self) -> bool {
            self.is_interactive
        }

        fn prompt_input(&self, _prompt: &str, _default: Option<&str>) -> Result<String> {
            let mut responses = self.prompt_responses.lock().unwrap();
            if responses.is_empty() {
                Ok(String::new())
            } else {
                Ok(responses.remove(0))
            }
        }

        fn prompt_select(&self, _prompt: &str, _items: &[&str], _default: usize) -> Result<usize> {
            Ok(0)
        }

        fn clear_screen(&self) {}
    }

    struct MockProgress;

    impl ProgressIndicator for MockProgress {
        fn set_message(&self, _message: &str) {}
        fn finish_and_clear(&self) {}
        fn enable_steady_tick(&self, _duration: std::time::Duration) {}
        fn finish_with_message(&self, _message: String) {}
        fn set_prefix(&self, _prefix: String) {}
    }

    struct MockMultiProgress;

    impl MultiProgressManager for MockMultiProgress {
        fn add_spinner(&self) -> Box<dyn ProgressIndicator> {
            Box::new(MockProgress)
        }
    }

    // Test helpers

    fn create_test_app(id: &str, name: &str, status: types::AppStatus) -> types::App {
        types::App {
            app_id: Uuid::parse_str(id).unwrap(),
            app_name: name.to_string(),
            status,
            provider_url: Some(format!("https://{name}.example.com")),
            provider_error: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    fn create_test_app_item(
        id: &str,
        name: &str,
        status: types::ListAppsResponseAppsItemStatus,
    ) -> types::ListAppsResponseAppsItem {
        types::ListAppsResponseAppsItem {
            app_id: Uuid::parse_str(id).unwrap(),
            app_name: name.to_string(),
            status,
            provider_url: Some(format!("https://{name}.example.com")),
            provider_error: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    // Tests

    #[tokio::test]
    async fn test_list_empty() {
        let api_client = MockApiClient {
            list_response: Ok(types::ListAppsResponse {
                apps: vec![],
                next_token: None,
            }),
            get_response: Err(anyhow::anyhow!("Not used")),
            delete_response: Err(anyhow::anyhow!("Not used")),
        };

        let ui = Arc::new(MockUI::new());
        let deps = Arc::new(EngDependencies {
            ui: ui.clone(),
            api_client: Arc::new(api_client),
        });

        let result = eng_cmd::list_with_deps(OutputFormat::Table, &deps).await;
        assert!(result.is_ok());

        let styled_messages = ui.get_styled_messages();
        assert_eq!(styled_messages.len(), 1);
        assert_eq!(styled_messages[0].0, "No engines found.");
        assert!(matches!(styled_messages[0].1, MessageStyle::Yellow));
    }

    #[tokio::test]
    async fn test_list_with_apps_table_format() {
        let api_client = MockApiClient {
            list_response: Ok(types::ListAppsResponse {
                apps: vec![
                    create_test_app_item(
                        "550e8400-e29b-41d4-a716-446655440000",
                        "test-app-1",
                        types::ListAppsResponseAppsItemStatus::Active,
                    ),
                    create_test_app_item(
                        "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
                        "test-app-2",
                        types::ListAppsResponseAppsItemStatus::Failed,
                    ),
                ],
                next_token: None,
            }),
            get_response: Err(anyhow::anyhow!("Not used")),
            delete_response: Err(anyhow::anyhow!("Not used")),
        };

        let ui = Arc::new(MockUI::new());
        let deps = Arc::new(EngDependencies {
            ui: ui.clone(),
            api_client: Arc::new(api_client),
        });

        let result = eng_cmd::list_with_deps(OutputFormat::Table, &deps).await;
        assert!(result.is_ok());

        let messages = ui.get_messages();
        // Should display app names, IDs, status, URLs
        assert!(messages.iter().any(|m| m.contains("test-app-1")));
        assert!(
            messages
                .iter()
                .any(|m| m.contains("550e8400-e29b-41d4-a716-446655440000"))
        );
        assert!(messages.iter().any(|m| m.contains("ACTIVE")));
        assert!(
            messages
                .iter()
                .any(|m| m.contains("https://test-app-1.example.com"))
        );
        assert!(messages.iter().any(|m| m.contains("Total: 2 engines")));
    }

    #[tokio::test]
    async fn test_list_with_apps_json_format() {
        let apps = vec![create_test_app_item(
            "550e8400-e29b-41d4-a716-446655440000",
            "test-app-1",
            types::ListAppsResponseAppsItemStatus::Active,
        )];

        let api_client = MockApiClient {
            list_response: Ok(types::ListAppsResponse {
                apps: apps.clone(),
                next_token: None,
            }),
            get_response: Err(anyhow::anyhow!("Not used")),
            delete_response: Err(anyhow::anyhow!("Not used")),
        };

        let ui = Arc::new(MockUI::new());
        let deps = Arc::new(EngDependencies {
            ui: ui.clone(),
            api_client: Arc::new(api_client),
        });

        let result = eng_cmd::list_with_deps(OutputFormat::Json, &deps).await;
        assert!(result.is_ok());

        let messages = ui.get_messages();
        assert_eq!(messages.len(), 1);

        // Verify JSON output
        let json_output = &messages[0];
        let parsed: serde_json::Value = serde_json::from_str(json_output).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_status_by_uuid() {
        let app_id = "550e8400-e29b-41d4-a716-446655440000";
        let app = create_test_app(app_id, "test-app", types::AppStatus::Active);

        let api_client = MockApiClient {
            list_response: Err(anyhow::anyhow!("Not used")),
            get_response: Ok(app),
            delete_response: Err(anyhow::anyhow!("Not used")),
        };

        let ui = Arc::new(MockUI::new());
        let deps = Arc::new(EngDependencies {
            ui: ui.clone(),
            api_client: Arc::new(api_client),
        });

        let result = eng_cmd::status_with_deps(app_id, OutputFormat::Table, &deps).await;
        assert!(result.is_ok());

        let messages = ui.get_messages();
        assert!(messages.iter().any(|m| m.contains("test-app")));
        assert!(messages.iter().any(|m| m.contains(app_id)));
        assert!(messages.iter().any(|m| m.contains("ACTIVE")));
    }

    #[tokio::test]
    async fn test_status_by_name() {
        let app_id = "550e8400-e29b-41d4-a716-446655440000";
        let app_name = "test-app";
        let app = create_test_app(app_id, app_name, types::AppStatus::Active);

        let api_client = MockApiClient {
            list_response: Ok(types::ListAppsResponse {
                apps: vec![create_test_app_item(
                    app_id,
                    app_name,
                    types::ListAppsResponseAppsItemStatus::Active,
                )],
                next_token: None,
            }),
            get_response: Ok(app),
            delete_response: Err(anyhow::anyhow!("Not used")),
        };

        let ui = Arc::new(MockUI::new());
        let deps = Arc::new(EngDependencies {
            ui: ui.clone(),
            api_client: Arc::new(api_client),
        });

        let result = eng_cmd::status_with_deps(app_name, OutputFormat::Table, &deps).await;
        assert!(result.is_ok());

        let messages = ui.get_messages();
        assert!(messages.iter().any(|m| m.contains(app_name)));
        assert!(messages.iter().any(|m| m.contains(app_id)));
    }

    #[tokio::test]
    async fn test_delete_with_confirmation() {
        let app_id = "550e8400-e29b-41d4-a716-446655440000";
        let app_name = "test-app";
        let app = create_test_app(app_id, app_name, types::AppStatus::Active);

        let api_client = MockApiClient {
            list_response: Err(anyhow::anyhow!("Not used")),
            get_response: Ok(app),
            delete_response: Ok(types::DeleteAppResponse {
                message: "Application deleted successfully".to_string(),
            }),
        };

        let ui = Arc::new(MockUI::new().with_prompt_response(app_name));
        let deps = Arc::new(EngDependencies {
            ui: ui.clone(),
            api_client: Arc::new(api_client),
        });

        let result = eng_cmd::delete_with_deps(app_id, false, &deps).await;
        assert!(result.is_ok());

        let styled_messages = ui.get_styled_messages();
        assert!(
            styled_messages
                .iter()
                .any(|(msg, _)| msg.contains("Application deleted successfully"))
        );
    }

    #[tokio::test]
    async fn test_delete_cancelled() {
        let app_id = "550e8400-e29b-41d4-a716-446655440000";
        let app_name = "test-app";
        let app = create_test_app(app_id, app_name, types::AppStatus::Active);

        let api_client = MockApiClient {
            list_response: Err(anyhow::anyhow!("Not used")),
            get_response: Ok(app),
            delete_response: Err(anyhow::anyhow!("Should not be called")),
        };

        let ui = Arc::new(MockUI::new().with_prompt_response("wrong-name"));
        let deps = Arc::new(EngDependencies {
            ui: ui.clone(),
            api_client: Arc::new(api_client),
        });

        let result = eng_cmd::delete_with_deps(app_id, false, &deps).await;
        assert!(result.is_ok());

        let styled_messages = ui.get_styled_messages();
        assert!(
            styled_messages
                .iter()
                .any(|(msg, _)| msg.contains("Deletion cancelled"))
        );
    }

    #[tokio::test]
    async fn test_delete_forced() {
        let app_id = "550e8400-e29b-41d4-a716-446655440000";
        let app_name = "test-app";
        let app = create_test_app(app_id, app_name, types::AppStatus::Active);

        let api_client = MockApiClient {
            list_response: Err(anyhow::anyhow!("Not used")),
            get_response: Ok(app),
            delete_response: Ok(types::DeleteAppResponse {
                message: "Application deleted successfully".to_string(),
            }),
        };

        let ui = Arc::new(MockUI::new());
        let deps = Arc::new(EngDependencies {
            ui: ui.clone(),
            api_client: Arc::new(api_client),
        });

        let result = eng_cmd::delete_with_deps(app_id, true, &deps).await;
        assert!(result.is_ok());

        // Should not prompt for confirmation
        let styled_messages = ui.get_styled_messages();
        assert!(
            styled_messages
                .iter()
                .any(|(msg, _)| msg.contains("Application deleted successfully"))
        );
    }
}
