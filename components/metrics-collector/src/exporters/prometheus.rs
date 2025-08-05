use std::collections::HashMap;

pub struct PrometheusExporter;

impl PrometheusExporter {
    pub fn format(metrics: &HashMap<String, serde_json::Value>) -> String {
        let mut output = String::new();
        
        // Add header
        output.push_str("# HELP ftl_tool_invocations_total The total number of tool invocations\n");
        output.push_str("# TYPE ftl_tool_invocations_total counter\n");
        
        // Process each tool's metrics
        for (tool_name, tool_metrics) in metrics {
            if tool_name == "_global" {
                // Handle global metrics
                if let Some(total) = tool_metrics.get("total_invocations") {
                    output.push_str(&format!("ftl_global_invocations_total {}\n", total));
                }
                continue;
            }
            
            // Tool-specific metrics
            if let Some(count) = tool_metrics.get("invocation_count") {
                output.push_str(&format!("ftl_tool_invocations_total{{tool=\"{}\"}} {}\n", tool_name, count));
            }
            
            if let Some(success) = tool_metrics.get("success_count") {
                output.push_str(&format!("ftl_tool_success_total{{tool=\"{}\"}} {}\n", tool_name, success));
            }
            
            if let Some(failure) = tool_metrics.get("failure_count") {
                output.push_str(&format!("ftl_tool_failures_total{{tool=\"{}\"}} {}\n", tool_name, failure));
            }
            
            if let Some(duration) = tool_metrics.get("total_duration_ms") {
                output.push_str(&format!("ftl_tool_duration_ms_total{{tool=\"{}\"}} {}\n", tool_name, duration));
            }
            
            if let Some(avg_duration) = tool_metrics.get("avg_duration_ms") {
                output.push_str(&format!("ftl_tool_duration_ms_avg{{tool=\"{}\"}} {}\n", tool_name, avg_duration));
            }
        }
        
        output
    }
}