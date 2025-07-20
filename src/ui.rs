//! User interface implementations

use std::sync::{Arc, Mutex};
use std::time::Duration;

use console::style;
use indicatif::{MultiProgress as IndicatifMultiProgress, ProgressBar, ProgressStyle};

use crate::deps::{MessageStyle, MultiProgressManager, ProgressIndicator, UserInterface};
use anyhow::Result;

/// Production UI implementation using indicatif
pub struct RealUserInterface;

impl UserInterface for RealUserInterface {
    fn create_spinner(&self) -> Box<dyn ProgressIndicator> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        Box::new(RealProgressIndicator { pb })
    }

    fn create_multi_progress(&self) -> Box<dyn MultiProgressManager> {
        Box::new(RealMultiProgressManager {
            mp: IndicatifMultiProgress::new(),
        })
    }

    fn print(&self, message: &str) {
        println!("{}", message);
    }

    fn print_styled(&self, message: &str, msg_style: MessageStyle) {
        let styled = match msg_style {
            MessageStyle::Normal => message.to_string(),
            MessageStyle::Bold => style(message).bold().to_string(),
            MessageStyle::Cyan => style(message).cyan().to_string(),
            MessageStyle::Green => style(message).green().to_string(),
            MessageStyle::Red => style(message).red().to_string(),
            MessageStyle::Yellow => style(message).yellow().to_string(),
            MessageStyle::Warning => style(message).yellow().bold().to_string(),
            MessageStyle::Error => style(message).red().bold().to_string(),
            MessageStyle::Success => style(message).green().bold().to_string(),
        };
        println!("{}", styled);
    }

    fn is_interactive(&self) -> bool {
        atty::is(atty::Stream::Stdin)
    }

    fn prompt_input(&self, prompt: &str, default: Option<&str>) -> Result<String> {
        use dialoguer::{Input, theme::ColorfulTheme};

        let theme = ColorfulTheme::default();
        let mut input = Input::<String>::with_theme(&theme).with_prompt(prompt);

        if let Some(default_val) = default {
            input = input.default(default_val.to_string());
        }

        input
            .interact_text()
            .map_err(|e| anyhow::anyhow!("Failed to get input: {}", e))
    }

    fn prompt_select(&self, prompt: &str, items: &[&str], default: usize) -> Result<usize> {
        use dialoguer::{Select, theme::ColorfulTheme};

        Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
            .map_err(|e| anyhow::anyhow!("Failed to get selection: {}", e))
    }

    fn clear_screen(&self) {
        print!("\x1B[2J\x1B[1;1H");
    }
}

struct RealProgressIndicator {
    pb: ProgressBar,
}

impl ProgressIndicator for RealProgressIndicator {
    fn set_message(&self, message: &str) {
        self.pb.set_message(message.to_string());
    }

    fn finish_and_clear(&self) {
        self.pb.finish_and_clear();
    }

    fn enable_steady_tick(&self, duration: Duration) {
        self.pb.enable_steady_tick(duration);
    }

    fn finish_with_message(&self, message: String) {
        self.pb.finish_with_message(message);
    }

    fn set_prefix(&self, prefix: String) {
        self.pb.set_prefix(prefix);
    }
}

struct RealMultiProgressManager {
    mp: IndicatifMultiProgress,
}

impl MultiProgressManager for RealMultiProgressManager {
    fn add_spinner(&self) -> Box<dyn ProgressIndicator> {
        let pb = self.mp.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {prefix:.bold} {msg}")
                .unwrap(),
        );
        Box::new(RealProgressIndicator { pb })
    }
}

// Test implementations for mocking

/// Test UI implementation that captures output
pub struct TestUserInterface {
    pub output: Arc<Mutex<Vec<String>>>,
    pub styled_output: Arc<Mutex<Vec<(String, MessageStyle)>>>,
}

impl TestUserInterface {
    pub fn new() -> Self {
        Self {
            output: Arc::new(Mutex::new(Vec::new())),
            styled_output: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_output(&self) -> Vec<String> {
        self.output.lock().unwrap().clone()
    }

    pub fn get_styled_output(&self) -> Vec<(String, MessageStyle)> {
        self.styled_output.lock().unwrap().clone()
    }
}

impl UserInterface for TestUserInterface {
    fn create_spinner(&self) -> Box<dyn ProgressIndicator> {
        Box::new(TestProgressIndicator {
            messages: Arc::new(Mutex::new(Vec::new())),
        })
    }

    fn create_multi_progress(&self) -> Box<dyn MultiProgressManager> {
        Box::new(TestMultiProgressManager)
    }

    fn print(&self, message: &str) {
        self.output.lock().unwrap().push(message.to_string());
    }

    fn print_styled(&self, message: &str, style: MessageStyle) {
        // Add to both styled output and regular output for easier testing
        self.styled_output
            .lock()
            .unwrap()
            .push((message.to_string(), style));
        self.output.lock().unwrap().push(message.to_string());
    }

    fn is_interactive(&self) -> bool {
        false // Test UI is non-interactive
    }

    fn prompt_input(&self, _prompt: &str, default: Option<&str>) -> Result<String> {
        // In test mode, return the default or a test value
        Ok(default.unwrap_or("test-value").to_string())
    }

    fn prompt_select(&self, _prompt: &str, _items: &[&str], default: usize) -> Result<usize> {
        // In test mode, return the default selection
        Ok(default)
    }

    fn clear_screen(&self) {
        // No-op in test mode
    }
}

struct TestProgressIndicator {
    messages: Arc<Mutex<Vec<String>>>,
}

impl ProgressIndicator for TestProgressIndicator {
    fn set_message(&self, message: &str) {
        self.messages.lock().unwrap().push(message.to_string());
    }

    fn finish_and_clear(&self) {}

    fn enable_steady_tick(&self, _duration: Duration) {}

    fn finish_with_message(&self, message: String) {
        self.messages.lock().unwrap().push(message);
    }

    fn set_prefix(&self, _prefix: String) {}
}

struct TestMultiProgressManager;

impl MultiProgressManager for TestMultiProgressManager {
    fn add_spinner(&self) -> Box<dyn ProgressIndicator> {
        Box::new(TestProgressIndicator {
            messages: Arc::new(Mutex::new(Vec::new())),
        })
    }
}
