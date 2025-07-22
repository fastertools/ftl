//! Tests for app commands

#[cfg(test)]
mod tests {
    #![allow(clippy::module_inception)]
    use std::sync::Arc;
    
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::DateTime;
    use uuid::Uuid;
    
    use crate::api_client::types::{
        self, DeleteAppResponse, DeleteAppResponseApp,
        GetAppStatusResponse, GetAppStatusResponseApp, GetAppStatusResponseAppLastDeploymentInfo,
        GetAppStatusResponseAppStatus, GetAppStatusResponseAppLastDeploymentInfoStatus,
        ListAppsResponse, ListAppsResponseAppsItem, ListAppsResponseAppsItemStatus,
        ListAppsResponsePagination,
    };
    use crate::commands::app::{self, AppDependencies, OutputFormat};
    use crate::deps::{
        FtlApiClient, MessageStyle, MultiProgressManager, ProgressIndicator,
        UserInterface,
    };
    
    // Mock implementations
    
    #[allow(clippy::struct_field_names)]
    struct MockApiClient {
        list_response: Result<ListAppsResponse>,
        status_response: Result<GetAppStatusResponse>,
        delete_response: Result<DeleteAppResponse>,
    }
    
    #[async_trait]
    impl FtlApiClient for MockApiClient {
        async fn get_ecr_credentials(&self) -> Result<types::GetEcrCredentialsResponse> {
            unimplemented!()
        }
        
        async fn create_ecr_repository(
            &self,
            _request: &types::CreateEcrRepositoryRequest,
        ) -> Result<types::CreateEcrRepositoryResponse> {
            unimplemented!()
        }
        
        async fn get_deployment_status(&self, _deployment_id: &str) -> Result<types::DeploymentStatus> {
            unimplemented!()
        }
        
        async fn deploy_app(
            &self,
            _request: &types::DeploymentRequest,
        ) -> Result<types::DeploymentResponse> {
            unimplemented!()
        }
        
        async fn list_apps(&self) -> Result<ListAppsResponse> {
            match &self.list_response {
                Ok(resp) => Ok(resp.clone()),
                Err(e) => Err(anyhow::anyhow!(e.to_string())),
            }
        }
        
        async fn get_app_status(&self, _app_name: &str) -> Result<GetAppStatusResponse> {
            match &self.status_response {
                Ok(resp) => Ok(resp.clone()),
                Err(e) => Err(anyhow::anyhow!(e.to_string())),
            }
        }
        
        async fn delete_app(&self, _app_name: &str) -> Result<DeleteAppResponse> {
            match &self.delete_response {
                Ok(resp) => Ok(resp.clone()),
                Err(e) => Err(anyhow::anyhow!(e.to_string())),
            }
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
            self.prompt_responses.lock().unwrap().push(response.to_string());
            self
        }
        
        fn non_interactive(mut self) -> Self {
            self.is_interactive = false;
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
            Box::new(MockProgressIndicator)
        }
        
        fn create_multi_progress(&self) -> Box<dyn MultiProgressManager> {
            unimplemented!()
        }
        
        fn print(&self, message: &str) {
            self.messages.lock().unwrap().push(message.to_string());
        }
        
        fn print_styled(&self, message: &str, style: MessageStyle) {
            self.styled_messages.lock().unwrap().push((message.to_string(), style));
        }
        
        fn is_interactive(&self) -> bool {
            self.is_interactive
        }
        
        fn prompt_input(&self, _prompt: &str, _default: Option<&str>) -> Result<String> {
            self.prompt_responses
                .lock()
                .unwrap()
                .pop()
                .ok_or_else(|| anyhow::anyhow!("No prompt response configured"))
        }
        
        fn prompt_select(&self, _prompt: &str, _items: &[&str], _default: usize) -> Result<usize> {
            unimplemented!()
        }
        
        fn clear_screen(&self) {}
    }
    
    struct MockProgressIndicator;
    
    impl ProgressIndicator for MockProgressIndicator {
        fn set_message(&self, _message: &str) {}
        fn finish_and_clear(&self) {}
        fn enable_steady_tick(&self, _duration: std::time::Duration) {}
        fn finish_with_message(&self, _message: String) {}
        fn set_prefix(&self, _prefix: String) {}
    }
    
    
    // Test helpers
    
    fn create_test_app(name: &str, status: &str) -> ListAppsResponseAppsItem {
        ListAppsResponseAppsItem {
            id: format!("{name}-id"),
            name: name.to_string(),
            url: format!("https://{name}.example.com"),
            urls: vec![format!("https://{name}.example.com")],
            status: Some(match status {
                "deployed" => ListAppsResponseAppsItemStatus::Deployed,
                "deploying" => ListAppsResponseAppsItemStatus::Deploying,
                "failed" => ListAppsResponseAppsItemStatus::Failed,
                _ => ListAppsResponseAppsItemStatus::Unknown,
            }),
            created_at: Some(DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into()),
            last_deployment: Some(DateTime::parse_from_rfc3339("2024-01-02T00:00:00Z").unwrap().into()),
            invocations: Some("100".to_string()),
        }
    }
    
    fn create_app_status_response(name: &str, status: &str) -> GetAppStatusResponse {
        GetAppStatusResponse {
            app: GetAppStatusResponseApp {
                id: format!("{name}-id"),
                name: name.to_string(),
                url: format!("https://{name}.example.com"),
                urls: vec![format!("https://{name}.example.com")],
                status: Some(match status {
                    "deployed" => GetAppStatusResponseAppStatus::Deployed,
                    "deploying" => GetAppStatusResponseAppStatus::Deploying,
                    "failed" => GetAppStatusResponseAppStatus::Failed,
                    _ => GetAppStatusResponseAppStatus::Unknown,
                }),
                created_at: Some(DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into()),
                last_deployment: Some(DateTime::parse_from_rfc3339("2024-01-02T00:00:00Z").unwrap().into()),
                invocations: Some("100".to_string()),
                deployment_count: 5,
                last_deployment_info: Some(GetAppStatusResponseAppLastDeploymentInfo {
                    deployment_id: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
                    status: GetAppStatusResponseAppLastDeploymentInfoStatus::Deployed,
                    created_at: DateTime::parse_from_rfc3339("2024-01-02T00:00:00Z").unwrap().into(),
                    completed_at: Some(DateTime::parse_from_rfc3339("2024-01-02T00:10:00Z").unwrap().into()),
                    deployment_url: Some("https://example.com/deployment".to_string()),
                    error: None,
                }),
            },
        }
    }
    
    // Tests
    
    #[tokio::test]
    async fn test_list_apps_empty() {
        let ui = Arc::new(MockUI::new());
        let api_client = Arc::new(MockApiClient {
            list_response: Ok(ListAppsResponse {
                apps: vec![],
                pagination: ListAppsResponsePagination {
                    has_more: false,
                    cursor: None,
                },
            }),
            status_response: Err(anyhow::anyhow!("not used")),
            delete_response: Err(anyhow::anyhow!("not used")),
        });
        
        let deps = Arc::new(AppDependencies {
            ui: ui.clone(),
            api_client,
        });
        
        let result = app::list_with_deps(OutputFormat::Table, &deps).await;
        assert!(result.is_ok());
        
        let styled = ui.get_styled_messages();
        assert!(styled.iter().any(|(msg, _)| msg.contains("No applications found")));
    }
    
    #[tokio::test]
    async fn test_list_apps_table() {
        let ui = Arc::new(MockUI::new());
        let api_client = Arc::new(MockApiClient {
            list_response: Ok(ListAppsResponse {
                apps: vec![
                    create_test_app("app1", "deployed"),
                    create_test_app("app2", "failed"),
                ],
                pagination: ListAppsResponsePagination {
                    has_more: false,
                    cursor: None,
                },
            }),
            status_response: Err(anyhow::anyhow!("not used")),
            delete_response: Err(anyhow::anyhow!("not used")),
        });
        
        let deps = Arc::new(AppDependencies {
            ui: ui.clone(),
            api_client,
        });
        
        let result = app::list_with_deps(OutputFormat::Table, &deps).await;
        assert!(result.is_ok());
        
        let messages = ui.get_messages();
        let styled = ui.get_styled_messages();
        
        // Check that app names are styled bold
        assert!(styled.iter().any(|(msg, style)| msg == "app1" && matches!(style, MessageStyle::Bold)));
        assert!(styled.iter().any(|(msg, style)| msg == "app2" && matches!(style, MessageStyle::Bold)));
        
        // Check URLs are displayed
        assert!(messages.iter().any(|msg| msg.contains("https://app1.example.com")));
        assert!(messages.iter().any(|msg| msg.contains("https://app2.example.com")));
        assert!(messages.iter().any(|msg| msg.contains("Total: 2 applications")));
    }
    
    #[tokio::test]
    async fn test_list_apps_json() {
        let ui = Arc::new(MockUI::new());
        let api_client = Arc::new(MockApiClient {
            list_response: Ok(ListAppsResponse {
                apps: vec![create_test_app("app1", "deployed")],
                pagination: ListAppsResponsePagination {
                    has_more: false,
                    cursor: None,
                },
            }),
            status_response: Err(anyhow::anyhow!("not used")),
            delete_response: Err(anyhow::anyhow!("not used")),
        });
        
        let deps = Arc::new(AppDependencies {
            ui: ui.clone(),
            api_client,
        });
        
        let result = app::list_with_deps(OutputFormat::Json, &deps).await;
        assert!(result.is_ok());
        
        let messages = ui.get_messages();
        let json_output = messages.iter().find(|msg| msg.contains("\"name\"")).unwrap();
        assert!(json_output.contains("\"name\": \"app1\""));
        assert!(json_output.contains("\"url\": \"https://app1.example.com\""));
    }
    
    #[tokio::test]
    async fn test_status_app() {
        let ui = Arc::new(MockUI::new());
        let api_client = Arc::new(MockApiClient {
            list_response: Err(anyhow::anyhow!("not used")),
            status_response: Ok(create_app_status_response("myapp", "deployed")),
            delete_response: Err(anyhow::anyhow!("not used")),
        });
        
        let deps = Arc::new(AppDependencies {
            ui: ui.clone(),
            api_client,
        });
        
        let result = app::status_with_deps("myapp", OutputFormat::Table, &deps).await;
        assert!(result.is_ok());
        
        let messages = ui.get_messages();
        let styled = ui.get_styled_messages();
        assert!(styled.iter().any(|(msg, _)| msg.contains("Application Details")));
        assert!(messages.iter().any(|msg| msg.contains("myapp")));
        assert!(messages.iter().any(|msg| msg.contains("Deployments:  5")));
        assert!(styled.iter().any(|(msg, _)| msg.contains("Last Deployment")));
    }
    
    #[tokio::test]
    async fn test_delete_app_with_confirmation() {
        let ui = Arc::new(MockUI::new().with_prompt_response("myapp"));
        let api_client = Arc::new(MockApiClient {
            list_response: Err(anyhow::anyhow!("not used")),
            status_response: Ok(create_app_status_response("myapp", "deployed")),
            delete_response: Ok(DeleteAppResponse {
                message: "Application deleted successfully".to_string(),
                app: DeleteAppResponseApp {
                    name: "myapp".to_string(),
                    deleted_at: DateTime::parse_from_rfc3339("2024-01-03T00:00:00Z").unwrap().into(),
                    last_deployment: None,
                },
                warning: "This action cannot be undone".to_string(),
            }),
        });
        
        let deps = Arc::new(AppDependencies {
            ui: ui.clone(),
            api_client,
        });
        
        let result = app::delete_with_deps("myapp", false, &deps).await;
        assert!(result.is_ok());
        
        let styled = ui.get_styled_messages();
        assert!(styled.iter().any(|(msg, _)| msg.contains("Application to be deleted")));
        
        let styled = ui.get_styled_messages();
        assert!(styled.iter().any(|(msg, _)| msg.contains("Application deleted successfully")));
        assert!(styled.iter().any(|(msg, style)| 
            msg.contains("This action cannot be undone") && matches!(style, MessageStyle::Warning)
        ));
    }
    
    #[tokio::test]
    async fn test_delete_app_cancelled() {
        let ui = Arc::new(MockUI::new().with_prompt_response("wrong-name"));
        let api_client = Arc::new(MockApiClient {
            list_response: Err(anyhow::anyhow!("not used")),
            status_response: Ok(create_app_status_response("myapp", "deployed")),
            delete_response: Err(anyhow::anyhow!("should not be called")),
        });
        
        let deps = Arc::new(AppDependencies {
            ui: ui.clone(),
            api_client,
        });
        
        let result = app::delete_with_deps("myapp", false, &deps).await;
        assert!(result.is_ok());
        
        let styled = ui.get_styled_messages();
        assert!(styled.iter().any(|(msg, _)| msg.contains("Deletion cancelled")));
    }
    
    #[tokio::test]
    async fn test_delete_app_force() {
        let ui = Arc::new(MockUI::new());
        let api_client = Arc::new(MockApiClient {
            list_response: Err(anyhow::anyhow!("not used")),
            status_response: Ok(create_app_status_response("myapp", "deployed")),
            delete_response: Ok(DeleteAppResponse {
                message: "Application deleted successfully".to_string(),
                app: DeleteAppResponseApp {
                    name: "myapp".to_string(),
                    deleted_at: DateTime::parse_from_rfc3339("2024-01-03T00:00:00Z").unwrap().into(),
                    last_deployment: None,
                },
                warning: String::new(),
            }),
        });
        
        let deps = Arc::new(AppDependencies {
            ui: ui.clone(),
            api_client,
        });
        
        let result = app::delete_with_deps("myapp", true, &deps).await;
        assert!(result.is_ok());
        
        // Should not see any prompts when force is true
        let messages = ui.get_messages();
        assert!(!messages.iter().any(|msg| msg.contains("Type 'myapp' to confirm")));
        
        let styled = ui.get_styled_messages();
        assert!(styled.iter().any(|(msg, _)| msg.contains("Application deleted successfully")));
    }
    
    #[tokio::test]
    async fn test_delete_app_non_interactive() {
        let ui = Arc::new(MockUI::new().non_interactive());
        let api_client = Arc::new(MockApiClient {
            list_response: Err(anyhow::anyhow!("not used")),
            status_response: Ok(create_app_status_response("myapp", "deployed")),
            delete_response: Ok(DeleteAppResponse {
                message: "Application deleted successfully".to_string(),
                app: DeleteAppResponseApp {
                    name: "myapp".to_string(),
                    deleted_at: DateTime::parse_from_rfc3339("2024-01-03T00:00:00Z").unwrap().into(),
                    last_deployment: None,
                },
                warning: String::new(),
            }),
        });
        
        let deps = Arc::new(AppDependencies {
            ui: ui.clone(),
            api_client,
        });
        
        let result = app::delete_with_deps("myapp", false, &deps).await;
        assert!(result.is_ok());
        
        // Should proceed without prompts in non-interactive mode
        let styled = ui.get_styled_messages();
        assert!(styled.iter().any(|(msg, _)| msg.contains("Application deleted successfully")));
    }
    
    #[tokio::test]
    async fn test_api_error_handling() {
        let ui = Arc::new(MockUI::new());
        let api_client = Arc::new(MockApiClient {
            list_response: Err(anyhow::anyhow!("API error: unauthorized")),
            status_response: Err(anyhow::anyhow!("not used")),
            delete_response: Err(anyhow::anyhow!("not used")),
        });
        
        let deps = Arc::new(AppDependencies {
            ui,
            api_client,
        });
        
        let result = app::list_with_deps(OutputFormat::Table, &deps).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API error: unauthorized"));
    }
}