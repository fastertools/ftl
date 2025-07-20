//! Tests for the Language enum

use crate::language::Language;

#[test]
fn test_language_from_str() {
    // Test exact matches
    assert_eq!(Language::from_str("rust"), Some(Language::Rust));
    assert_eq!(Language::from_str("javascript"), Some(Language::JavaScript));
    assert_eq!(Language::from_str("typescript"), Some(Language::TypeScript));

    // Test aliases
    assert_eq!(Language::from_str("rs"), Some(Language::Rust));
    assert_eq!(Language::from_str("js"), Some(Language::JavaScript));
    assert_eq!(Language::from_str("ts"), Some(Language::TypeScript));

    // Test case insensitive
    assert_eq!(Language::from_str("RUST"), Some(Language::Rust));
    assert_eq!(Language::from_str("JavaScript"), Some(Language::JavaScript));
    assert_eq!(Language::from_str("TypeScript"), Some(Language::TypeScript));

    // Test unknown
    assert_eq!(Language::from_str("python"), None);
    assert_eq!(Language::from_str(""), None);
}

#[test]
fn test_language_display() {
    assert_eq!(Language::Rust.to_string(), "Rust");
    assert_eq!(Language::JavaScript.to_string(), "JavaScript");
    assert_eq!(Language::TypeScript.to_string(), "TypeScript");
}

#[test]
fn test_language_default() {
    let default_lang: Language = Default::default();
    assert_eq!(default_lang, Language::TypeScript);
}
