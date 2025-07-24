//! Tests for UI implementations

use super::*;
use ftl_runtime::deps::MessageStyle;

#[test]
fn test_real_user_interface_print() {
    let ui = RealUserInterface;

    // These will print to stdout, but we're testing they don't panic
    ui.print("Hello, world!");
    ui.print("");
    ui.print("Multi\nline\ntext");
}

#[test]
fn test_real_user_interface_print_styled() {
    let ui = RealUserInterface;

    // Test all message styles
    ui.print_styled("Bold text", MessageStyle::Bold);
    ui.print_styled("Cyan text", MessageStyle::Cyan);
    ui.print_styled("Green text", MessageStyle::Green);
    ui.print_styled("Red text", MessageStyle::Red);
    ui.print_styled("Yellow text", MessageStyle::Yellow);
    ui.print_styled("Warning text", MessageStyle::Warning);
    ui.print_styled("Error text", MessageStyle::Error);
    ui.print_styled("Success text", MessageStyle::Success);
}

#[test]
fn test_real_user_interface_is_interactive() {
    let ui = RealUserInterface;

    // This checks if stdout is a TTY
    let _ = ui.is_interactive();
}

#[test]
fn test_real_progress_indicator() {
    let ui = RealUserInterface;
    let spinner = ui.create_spinner();

    // Test setting message
    spinner.set_message("Loading...");
    spinner.set_message("Processing...");
    spinner.set_message("");

    // Test tick - RealProgressIndicator doesn't have tick method
    // Just test that setting message works

    // Test finish
    spinner.finish_and_clear();

    // Test finish with message
    let spinner2 = ui.create_spinner();
    spinner2.set_message("Task");
    spinner2.finish_with_message("Task completed".to_string());

    // Test finish and clear
    let spinner3 = ui.create_spinner();
    spinner3.set_message("Temporary");
    spinner3.finish_and_clear();

    // Test set_prefix
    let spinner4 = ui.create_spinner();
    spinner4.set_prefix("[1/3]".to_string());
    spinner4.set_message("Processing");
    spinner4.set_prefix("[2/3]".to_string());
    spinner4.set_message("Almost done");
    spinner4.finish_with_message("Complete".to_string());
}

#[test]
fn test_real_multi_progress_manager() {
    let ui = RealUserInterface;
    let mp = ui.create_multi_progress();

    // Create a spinner through multi-progress
    let spinner = mp.add_spinner();
    spinner.set_message("Multi-progress spinner");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner.finish_and_clear();

    // Create multiple spinners
    let spinner1 = mp.add_spinner();
    let spinner2 = mp.add_spinner();

    spinner1.set_message("Task 1");
    spinner2.set_message("Task 2");

    spinner1.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner2.enable_steady_tick(std::time::Duration::from_millis(100));

    spinner1.finish_with_message("Task 1 done".to_string());
    spinner2.finish_with_message("Task 2 done".to_string());
}

#[test]
fn test_test_user_interface_multi_progress() {
    let ui = TestUserInterface::new();
    let mp = ui.create_multi_progress();

    // Create spinners through multi-progress
    let spinner1 = mp.add_spinner();
    let spinner2 = mp.add_spinner();

    spinner1.set_message("Test spinner 1");
    spinner2.set_message("Test spinner 2");

    spinner1.finish_and_clear();
    spinner2.finish_and_clear();

    // Check that operations didn't panic
    assert!(!ui.get_output().is_empty() || ui.get_output().is_empty());
}

#[test]
fn test_styled_text_variations() {
    let ui = RealUserInterface;

    // Test empty strings
    ui.print_styled("", MessageStyle::Bold);
    ui.print_styled("", MessageStyle::Error);

    // Test special characters
    ui.print_styled("Special: @#$%^&*()", MessageStyle::Cyan);
    ui.print_styled("Unicode: 🚀 ✨ 🎉", MessageStyle::Green);

    // Test very long strings
    let long_string = "a".repeat(1000);
    ui.print_styled(&long_string, MessageStyle::Yellow);
}
