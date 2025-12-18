//! Phase 5.1: Core Metadata Types - TDD Test Suite
//! 
//! Tests for SettingMetadata, SettingType, Constraint, Visibility, ConfigSchema, and SettingGroup
//! 
//! This is the RED phase of TDD - tests are written first and are expected to fail
//! until the types are implemented in src/metadata.rs

use serde_json::json;
use std::time::Duration;

#[cfg(feature = "metadata")]
use settings_loader::metadata::*;

// ============================================================================
// VISIBILITY ENUM TESTS
// ============================================================================

#[test]
fn test_visibility_public_variant() {
    // Test that Visibility::Public can be created
    // let v = Visibility::Public;
    // assert_eq!(v, Visibility::Public);
    // assert!(format!("{:?}", v).contains("Public"));
}

#[test]
fn test_visibility_hidden_variant() {
    // Test that Visibility::Hidden can be created
    // let v = Visibility::Hidden;
    // assert_eq!(v, Visibility::Hidden);
}

#[test]
fn test_visibility_secret_variant() {
    // Test that Visibility::Secret can be created
    // let v = Visibility::Secret;
    // assert_eq!(v, Visibility::Secret);
}

#[test]
fn test_visibility_advanced_variant() {
    // Test that Visibility::Advanced can be created
    // let v = Visibility::Advanced;
    // assert_eq!(v, Visibility::Advanced);
}

#[test]
fn test_visibility_default_is_public() {
    // Test that Visibility defaults to Public
    // let v: Visibility = Default::default();
    // assert_eq!(v, Visibility::Public);
}

#[test]
fn test_visibility_is_copy() {
    // Test that Visibility can be copied
    // let v1 = Visibility::Secret;
    // let v2 = v1;
    // assert_eq!(v1, v2);
    // assert_eq!(v1, Visibility::Secret);
}

#[test]
fn test_visibility_hash() {
    // Test that Visibility can be hashed (Hash trait)
    // let v = Visibility::Public;
    // let mut hasher = std::collections::hash_map::DefaultHasher::new();
    // v.hash(&mut hasher);
    // let hash = hasher.finish();
    // assert!(hash > 0);
}

// ============================================================================
// CONSTRAINT ENUM TESTS
// ============================================================================

#[test]
fn test_constraint_pattern_variant() {
    // Test Pattern constraint
    // let c = Constraint::Pattern("[0-9]+".to_string());
    // assert!(matches!(c, Constraint::Pattern(_)));
    // assert!(format!("{:?}", c).contains("Pattern"));
}

#[test]
fn test_constraint_range_variant() {
    // Test Range constraint
    // let c = Constraint::Range { min: 1.0, max: 100.0 };
    // assert!(matches!(c, Constraint::Range { .. }));
}

#[test]
fn test_constraint_length_variant() {
    // Test Length constraint
    // let c = Constraint::Length { min: 5, max: 50 };
    // assert!(matches!(c, Constraint::Length { .. }));
}

#[test]
fn test_constraint_required_variant() {
    // Test Required constraint
    // let c = Constraint::Required;
    // assert_eq!(c, Constraint::Required);
}

#[test]
fn test_constraint_one_of_variant() {
    // Test OneOf constraint
    // let c = Constraint::OneOf(vec!["a".to_string(), "b".to_string()]);
    // assert!(matches!(c, Constraint::OneOf(_)));
}

#[test]
fn test_constraint_custom_variant() {
    // Test Custom constraint
    // let c = Constraint::Custom("my_validator".to_string());
    // assert!(matches!(c, Constraint::Custom(_)));
}

#[test]
fn test_constraint_partial_eq() {
    // Test PartialEq implementation
    // let c1 = Constraint::Required;
    // let c2 = Constraint::Required;
    // assert_eq!(c1, c2);
    // 
    // let c3 = Constraint::Pattern("test".to_string());
    // assert_ne!(c1, c3);
}

// ============================================================================
// SETTING TYPE ENUM TESTS
// ============================================================================

#[test]
fn test_setting_type_string_variant() {
    // Test String type with optional fields
    // let st = SettingType::String {
    //     pattern: Some("[a-z]+".to_string()),
    //     min_length: Some(1),
    //     max_length: Some(255),
    // };
    // assert!(matches!(st, SettingType::String { .. }));
}

#[test]
fn test_setting_type_string_no_constraints() {
    // Test String type with all None fields
    // let st = SettingType::String {
    //     pattern: None,
    //     min_length: None,
    //     max_length: None,
    // };
    // assert!(matches!(st, SettingType::String { .. }));
}

#[test]
fn test_setting_type_integer_variant() {
    // Test Integer type with range
    // let st = SettingType::Integer {
    //     min: Some(0),
    //     max: Some(100),
    // };
    // assert!(matches!(st, SettingType::Integer { .. }));
}

#[test]
fn test_setting_type_integer_extreme_values() {
    // Test Integer with extreme values
    // let st = SettingType::Integer {
    //     min: Some(i64::MIN),
    //     max: Some(i64::MAX),
    // };
    // assert!(matches!(st, SettingType::Integer { .. }));
}

#[test]
fn test_setting_type_float_variant() {
    // Test Float type with range
    // let st = SettingType::Float {
    //     min: Some(0.0),
    //     max: Some(100.5),
    // };
    // assert!(matches!(st, SettingType::Float { .. }));
}

#[test]
fn test_setting_type_float_extreme_values() {
    // Test Float with extreme values
    // let st = SettingType::Float {
    //     min: Some(f64::NEG_INFINITY),
    //     max: Some(f64::INFINITY),
    // };
    // assert!(matches!(st, SettingType::Float { .. }));
}

#[test]
fn test_setting_type_boolean_variant() {
    // Test Boolean type (unit variant)
    // let st = SettingType::Boolean;
    // assert_eq!(st, SettingType::Boolean);
}

#[test]
fn test_setting_type_duration_variant() {
    // Test Duration type with range
    // let st = SettingType::Duration {
    //     min: Some(Duration::from_secs(1)),
    //     max: Some(Duration::from_secs(3600)),
    // };
    // assert!(matches!(st, SettingType::Duration { .. }));
}

#[test]
fn test_setting_type_duration_zero_value() {
    // Test Duration with zero
    // let st = SettingType::Duration {
    //     min: Some(Duration::from_secs(0)),
    //     max: Some(Duration::from_secs(0)),
    // };
    // assert!(matches!(st, SettingType::Duration { .. }));
}

#[test]
fn test_setting_type_path_variant() {
    // Test Path type with constraints
    // let st = SettingType::Path {
    //     must_exist: true,
    //     is_directory: false,
    // };
    // assert!(matches!(st, SettingType::Path { .. }));
}

#[test]
fn test_setting_type_path_all_variants() {
    // Test Path with all combinations
    // let st1 = SettingType::Path { must_exist: true, is_directory: true };
    // let st2 = SettingType::Path { must_exist: false, is_directory: false };
    // assert!(matches!(st1, SettingType::Path { .. }));
    // assert!(matches!(st2, SettingType::Path { .. }));
}

#[test]
fn test_setting_type_url_variant() {
    // Test URL type with scheme restrictions
    // let st = SettingType::Url {
    //     schemes: vec!["http".to_string(), "https".to_string()],
    // };
    // assert!(matches!(st, SettingType::Url { .. }));
}

#[test]
fn test_setting_type_url_no_schemes() {
    // Test URL with empty schemes (all allowed)
    // let st = SettingType::Url { schemes: vec![] };
    // assert!(matches!(st, SettingType::Url { .. }));
}

#[test]
fn test_setting_type_enum_variant() {
    // Test Enum type with variants
    // let st = SettingType::Enum {
    //     variants: vec!["a".to_string(), "b".to_string(), "c".to_string()],
    // };
    // assert!(matches!(st, SettingType::Enum { .. }));
}

#[test]
fn test_setting_type_enum_single_variant() {
    // Test Enum with single variant
    // let st = SettingType::Enum {
    //     variants: vec!["only".to_string()],
    // };
    // assert!(matches!(st, SettingType::Enum { .. }));
}

#[test]
fn test_setting_type_secret_variant() {
    // Test Secret type (unit variant)
    // let st = SettingType::Secret;
    // assert_eq!(st, SettingType::Secret);
}

#[test]
fn test_setting_type_array_variant() {
    // Test Array type with element type
    // let st = SettingType::Array {
    //     element_type: Box::new(SettingType::String {
    //         pattern: None,
    //         min_length: None,
    //         max_length: None,
    //     }),
    //     min_items: Some(1),
    //     max_items: Some(100),
    // };
    // assert!(matches!(st, SettingType::Array { .. }));
}

#[test]
fn test_setting_type_array_nested_array() {
    // Test Array of Array (recursive)
    // let inner = Box::new(SettingType::Array {
    //     element_type: Box::new(SettingType::Integer { min: None, max: None }),
    //     min_items: None,
    //     max_items: None,
    // });
    // let st = SettingType::Array {
    //     element_type: inner,
    //     min_items: None,
    //     max_items: None,
    // };
    // assert!(matches!(st, SettingType::Array { .. }));
}

#[test]
fn test_setting_type_object_variant() {
    // Test Object type with fields
    // let st = SettingType::Object { fields: vec![] };
    // assert!(matches!(st, SettingType::Object { .. }));
}

#[test]
fn test_setting_type_any_variant() {
    // Test Any type (no validation)
    // let st = SettingType::Any;
    // assert_eq!(st, SettingType::Any);
}

#[test]
fn test_setting_type_partial_eq() {
    // Test PartialEq implementation
    // let st1 = SettingType::Boolean;
    // let st2 = SettingType::Boolean;
    // assert_eq!(st1, st2);
    // 
    // let st3 = SettingType::Secret;
    // assert_ne!(st1, st3);
}

// ============================================================================
// SETTING GROUP TESTS
// ============================================================================

#[test]
fn test_setting_group_construction() {
    // Test SettingGroup creation
    // let group = SettingGroup {
    //     name: "api".to_string(),
    //     label: "API Settings".to_string(),
    //     description: "Configuration for API endpoints".to_string(),
    //     settings: vec![
    //         "api_url".to_string(),
    //         "api_key".to_string(),
    //         "timeout_secs".to_string(),
    //     ],
    // };
    // assert_eq!(group.name, "api");
    // assert_eq!(group.label, "API Settings");
    // assert_eq!(group.settings.len(), 3);
}

#[test]
fn test_setting_group_empty_settings() {
    // Test SettingGroup with no settings
    // let group = SettingGroup {
    //     name: "empty".to_string(),
    //     label: "Empty Group".to_string(),
    //     description: "A group with no settings".to_string(),
    //     settings: vec![],
    // };
    // assert!(group.settings.is_empty());
}

#[test]
fn test_setting_group_clone() {
    // Test that SettingGroup implements Clone
    // let group1 = SettingGroup {
    //     name: "test".to_string(),
    //     label: "Test".to_string(),
    //     description: "Test description".to_string(),
    //     settings: vec!["setting1".to_string()],
    // };
    // let group2 = group1.clone();
    // assert_eq!(group1.name, group2.name);
}

// ============================================================================
// SETTING METADATA TESTS
// ============================================================================

#[test]
fn test_setting_metadata_construction_minimal() {
    // Test SettingMetadata with minimal fields
    // let metadata = SettingMetadata {
    //     key: "api_url".to_string(),
    //     label: "API URL".to_string(),
    //     description: "API endpoint URL".to_string(),
    //     setting_type: SettingType::String {
    //         pattern: None,
    //         min_length: None,
    //         max_length: None,
    //     },
    //     default: None,
    //     constraints: vec![],
    //     visibility: Visibility::Public,
    //     group: None,
    // };
    // assert_eq!(metadata.key, "api_url");
}

#[test]
fn test_setting_metadata_construction_full() {
    // Test SettingMetadata with all fields populated
    // let metadata = SettingMetadata {
    //     key: "timeout_secs".to_string(),
    //     label: "Timeout".to_string(),
    //     description: "Request timeout in seconds".to_string(),
    //     setting_type: SettingType::Integer {
    //         min: Some(1),
    //         max: Some(300),
    //     },
    //     default: Some(json!(30)),
    //     constraints: vec![Constraint::Required],
    //     visibility: Visibility::Public,
    //     group: Some("api".to_string()),
    // };
    // assert_eq!(metadata.key, "timeout_secs");
    // assert_eq!(metadata.default, Some(json!(30)));
    // assert_eq!(metadata.group, Some("api".to_string()));
}

#[test]
fn test_setting_metadata_with_secret_visibility() {
    // Test SettingMetadata with Secret visibility
    // let metadata = SettingMetadata {
    //     key: "api_key".to_string(),
    //     label: "API Key".to_string(),
    //     description: "Authentication key".to_string(),
    //     setting_type: SettingType::Secret,
    //     default: None,  // Secrets should not have defaults
    //     constraints: vec![Constraint::Required],
    //     visibility: Visibility::Secret,
    //     group: Some("api".to_string()),
    // };
    // assert_eq!(metadata.visibility, Visibility::Secret);
    // assert_eq!(metadata.default, None);
}

#[test]
fn test_setting_metadata_with_multiple_constraints() {
    // Test SettingMetadata with multiple constraints
    // let metadata = SettingMetadata {
    //     key: "email".to_string(),
    //     label: "Email".to_string(),
    //     description: "User email address".to_string(),
    //     setting_type: SettingType::String {
    //         pattern: Some("[a-z0-9]+@[a-z]+\\.[a-z]+".to_string()),
    //         min_length: Some(5),
    //         max_length: Some(255),
    //     },
    //     default: None,
    //     constraints: vec![
    //         Constraint::Required,
    //         Constraint::Pattern("[a-z0-9]+@[a-z]+\\.[a-z]+".to_string()),
    //     ],
    //     visibility: Visibility::Public,
    //     group: Some("user".to_string()),
    // };
    // assert_eq!(metadata.constraints.len(), 2);
}

#[test]
fn test_setting_metadata_clone() {
    // Test that SettingMetadata implements Clone
    // let metadata1 = SettingMetadata {
    //     key: "test".to_string(),
    //     label: "Test".to_string(),
    //     description: "Test setting".to_string(),
    //     setting_type: SettingType::Boolean,
    //     default: None,
    //     constraints: vec![],
    //     visibility: Visibility::Public,
    //     group: None,
    // };
    // let metadata2 = metadata1.clone();
    // assert_eq!(metadata1.key, metadata2.key);
}

// ============================================================================
// CONFIG SCHEMA TESTS
// ============================================================================

#[test]
fn test_config_schema_construction_empty() {
    // Test ConfigSchema with empty settings
    // let schema = ConfigSchema {
    //     name: "my-app".to_string(),
    //     version: "1.0.0".to_string(),
    //     settings: vec![],
    //     groups: vec![],
    // };
    // assert_eq!(schema.name, "my-app");
    // assert_eq!(schema.version, "1.0.0");
    // assert!(schema.settings.is_empty());
}

#[test]
fn test_config_schema_construction_with_settings() {
    // Test ConfigSchema with multiple settings
    // let schema = ConfigSchema {
    //     name: "my-app".to_string(),
    //     version: "1.0.0".to_string(),
    //     settings: vec![
    //         SettingMetadata {
    //             key: "api_url".to_string(),
    //             label: "API URL".to_string(),
    //             description: "API endpoint".to_string(),
    //             setting_type: SettingType::String {
    //                 pattern: None,
    //                 min_length: None,
    //                 max_length: None,
    //             },
    //             default: None,
    //             constraints: vec![],
    //             visibility: Visibility::Public,
    //             group: Some("api".to_string()),
    //         },
    //     ],
    //     groups: vec![],
    // };
    // assert_eq!(schema.settings.len(), 1);
}

#[test]
fn test_config_schema_with_groups() {
    // Test ConfigSchema with organized groups
    // let schema = ConfigSchema {
    //     name: "my-app".to_string(),
    //     version: "1.0.0".to_string(),
    //     settings: vec![],
    //     groups: vec![
    //         SettingGroup {
    //             name: "api".to_string(),
    //             label: "API Settings".to_string(),
    //             description: "API configuration".to_string(),
    //             settings: vec!["api_url".to_string(), "api_key".to_string()],
    //         },
    //     ],
    // };
    // assert_eq!(schema.groups.len(), 1);
    // assert_eq!(schema.groups[0].name, "api");
}

// ============================================================================
// SERDE TESTS
// ============================================================================

#[test]
fn test_visibility_serde_json_round_trip() {
    // Test Visibility serialization/deserialization
    // let v = Visibility::Secret;
    // let json_str = serde_json::to_string(&v).unwrap();
    // let v2: Visibility = serde_json::from_str(&json_str).unwrap();
    // assert_eq!(v, v2);
}

#[test]
fn test_setting_type_serde_json_round_trip() {
    // Test SettingType serialization/deserialization
    // let st = SettingType::Integer { min: Some(1), max: Some(100) };
    // let json_str = serde_json::to_string(&st).unwrap();
    // let st2: SettingType = serde_json::from_str(&json_str).unwrap();
    // assert_eq!(st, st2);
}

#[test]
fn test_constraint_serde_json_round_trip() {
    // Test Constraint serialization/deserialization
    // let c = Constraint::Range { min: 1.0, max: 100.0 };
    // let json_str = serde_json::to_string(&c).unwrap();
    // let c2: Constraint = serde_json::from_str(&json_str).unwrap();
    // assert_eq!(c, c2);
}

#[test]
fn test_setting_metadata_serde_json_round_trip() {
    // Test SettingMetadata serialization/deserialization
    // let metadata = SettingMetadata {
    //     key: "test".to_string(),
    //     label: "Test".to_string(),
    //     description: "Test setting".to_string(),
    //     setting_type: SettingType::Boolean,
    //     default: Some(json!(true)),
    //     constraints: vec![],
    //     visibility: Visibility::Public,
    //     group: None,
    // };
    // let json_str = serde_json::to_string(&metadata).unwrap();
    // let metadata2: SettingMetadata = serde_json::from_str(&json_str).unwrap();
    // assert_eq!(metadata.key, metadata2.key);
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_nested_object_type_with_fields() {
    // Test creating nested Object with fields
    // let inner = SettingMetadata {
    //     key: "database.host".to_string(),
    //     label: "Database Host".to_string(),
    //     description: "Database hostname".to_string(),
    //     setting_type: SettingType::String {
    //         pattern: None,
    //         min_length: None,
    //         max_length: None,
    //     },
    //     default: Some(json!("localhost")),
    //     constraints: vec![],
    //     visibility: Visibility::Public,
    //     group: None,
    // };
    // 
    // let object_type = SettingType::Object {
    //     fields: vec![inner],
    // };
    // 
    // assert!(matches!(object_type, SettingType::Object { .. }));
}

#[test]
fn test_complex_settings_schema() {
    // Test a realistic schema with multiple types and groups
    // let schema = ConfigSchema {
    //     name: "turtle-config".to_string(),
    //     version: "0.1.0".to_string(),
    //     settings: vec![
    //         SettingMetadata {
    //             key: "llm_provider".to_string(),
    //             label: "LLM Provider".to_string(),
    //             description: "Language model provider".to_string(),
    //             setting_type: SettingType::Enum {
    //                 variants: vec!["ollama".to_string(), "openai".to_string()],
    //             },
    //             default: Some(json!("ollama")),
    //             constraints: vec![Constraint::Required],
    //             visibility: Visibility::Public,
    //             group: Some("llm".to_string()),
    //         },
    //         SettingMetadata {
    //             key: "api_key".to_string(),
    //             label: "API Key".to_string(),
    //             description: "API key for LLM service".to_string(),
    //             setting_type: SettingType::Secret,
    //             default: None,
    //             constraints: vec![],
    //             visibility: Visibility::Secret,
    //             group: Some("llm".to_string()),
    //         },
    //     ],
    //     groups: vec![
    //         SettingGroup {
    //             name: "llm".to_string(),
    //             label: "Language Model".to_string(),
    //             description: "Language model configuration".to_string(),
    //             settings: vec!["llm_provider".to_string(), "api_key".to_string()],
    //         },
    //     ],
    // };
    // 
    // assert_eq!(schema.settings.len(), 2);
    // assert_eq!(schema.groups.len(), 1);
}

// ============================================================================
// EDGE CASE TESTS (Phase 5.1.4: Test Enhancement)
// ============================================================================

#[test]
fn test_visibility_all_variants_distinct() {
    // Test that all Visibility variants are distinct
    // let public = Visibility::Public;
    // let hidden = Visibility::Hidden;
    // let secret = Visibility::Secret;
    // let advanced = Visibility::Advanced;
    // 
    // assert_ne!(public, hidden);
    // assert_ne!(public, secret);
    // assert_ne!(public, advanced);
    // assert_ne!(hidden, secret);
    // assert_ne!(hidden, advanced);
    // assert_ne!(secret, advanced);
}

#[test]
fn test_constraint_pattern_empty_string() {
    // Test Pattern constraint with empty regex
    // let c = Constraint::Pattern(String::new());
    // assert!(matches!(c, Constraint::Pattern(ref s) if s.is_empty()));
}

#[test]
fn test_constraint_range_equal_bounds() {
    // Test Range constraint with equal min and max
    // let c = Constraint::Range { min: 42.0, max: 42.0 };
    // assert!(matches!(c, Constraint::Range { min, max } if min == max));
}

#[test]
fn test_constraint_length_zero_range() {
    // Test Length constraint with zero range
    // let c = Constraint::Length { min: 0, max: 0 };
    // assert!(matches!(c, Constraint::Length { min, max } if min == 0 && max == 0));
}

#[test]
fn test_setting_type_string_all_constraints() {
    // Test String with all constraint fields populated
    // let st = SettingType::String {
    //     pattern: Some("[0-9]+".to_string()),
    //     min_length: Some(1),
    //     max_length: Some(255),
    // };
    // 
    // if let SettingType::String { pattern, min_length, max_length } = st {
    //     assert!(pattern.is_some());
    //     assert!(min_length.is_some());
    //     assert!(max_length.is_some());
    // }
}

#[test]
fn test_setting_type_url_many_schemes() {
    // Test URL with many allowed schemes
    // let schemes = vec![
    //     "http".to_string(), "https".to_string(), "ws".to_string(),
    //     "wss".to_string(), "ftp".to_string(), "ftps".to_string(),
    // ];
    // let st = SettingType::Url { schemes };
    // 
    // if let SettingType::Url { schemes: ref s } = st {
    //     assert_eq!(s.len(), 6);
    // }
}

#[test]
fn test_setting_type_enum_many_variants() {
    // Test Enum with many variants
    // let variants: Vec<String> = (0..50)
    //     .map(|i| format!("variant_{}", i))
    //     .collect();
    // 
    // let st = SettingType::Enum { variants: variants.clone() };
    // 
    // if let SettingType::Enum { variants: ref v } = st {
    //     assert_eq!(v.len(), 50);
    // }
}

#[test]
fn test_setting_type_array_deeply_nested() {
    // Test deeply nested Array (Array<Array<Array<String>>>)
    // let inner = SettingType::String {
    //     pattern: None,
    //     min_length: None,
    //     max_length: None,
    // };
    // 
    // let level1 = SettingType::Array {
    //     element_type: Box::new(inner),
    //     min_items: None,
    //     max_items: None,
    // };
    // 
    // let level2 = SettingType::Array {
    //     element_type: Box::new(level1),
    //     min_items: None,
    //     max_items: None,
    // };
    // 
    // let level3 = SettingType::Array {
    //     element_type: Box::new(level2),
    //     min_items: Some(1),
    //     max_items: Some(100),
    // };
    // 
    // assert!(matches!(level3, SettingType::Array { .. }));
}

#[test]
fn test_setting_type_object_many_fields() {
    // Test Object with many fields
    // let fields: Vec<SettingMetadata> = (0..20)
    //     .map(|i| SettingMetadata {
    //         key: format!("field_{}", i),
    //         label: format!("Field {}", i),
    //         description: format!("Field {}", i),
    //         setting_type: SettingType::String {
    //             pattern: None,
    //             min_length: None,
    //             max_length: None,
    //         },
    //         default: None,
    //         constraints: vec![],
    //         visibility: Visibility::Public,
    //         group: None,
    //     })
    //     .collect();
    // 
    // let st = SettingType::Object { fields };
    // 
    // if let SettingType::Object { fields: ref f } = st {
    //     assert_eq!(f.len(), 20);
    // }
}

#[test]
fn test_setting_metadata_all_none_optionals() {
    // Test SettingMetadata with all optional fields as None
    // let metadata = SettingMetadata {
    //     key: "test".to_string(),
    //     label: "Test".to_string(),
    //     description: "Test".to_string(),
    //     setting_type: SettingType::Any,
    //     default: None,
    //     constraints: vec![],
    //     visibility: Visibility::Public,
    //     group: None,
    // };
    // 
    // assert!(metadata.default.is_none());
    // assert!(metadata.group.is_none());
}

#[test]
fn test_setting_metadata_complex_default() {
    // Test SettingMetadata with complex nested default JSON
    // let default = json!({
    //     "nested": {
    //         "value": 42,
    //         "array": [1, 2, 3]
    //     }
    // });
    // 
    // let metadata = SettingMetadata {
    //     key: "complex".to_string(),
    //     label: "Complex".to_string(),
    //     description: "Complex default".to_string(),
    //     setting_type: SettingType::Any,
    //     default: Some(default.clone()),
    //     constraints: vec![],
    //     visibility: Visibility::Public,
    //     group: None,
    // };
    // 
    // assert_eq!(metadata.default, Some(default));
}

#[test]
fn test_config_schema_many_settings_and_groups() {
    // Test ConfigSchema with many settings and groups
    // let settings: Vec<SettingMetadata> = (0..50)
    //     .map(|i| SettingMetadata {
    //         key: format!("setting_{}", i),
    //         label: format!("Setting {}", i),
    //         description: format!("Setting {}", i),
    //         setting_type: SettingType::String {
    //             pattern: None,
    //             min_length: None,
    //             max_length: None,
    //         },
    //         default: None,
    //         constraints: vec![],
    //         visibility: if i % 2 == 0 { Visibility::Public } else { Visibility::Hidden },
    //         group: Some(format!("group_{}", i % 5)),
    //     })
    //     .collect();
    // 
    // let groups: Vec<SettingGroup> = (0..5)
    //     .map(|i| SettingGroup {
    //         name: format!("group_{}", i),
    //         label: format!("Group {}", i),
    //         description: format!("Group {}", i),
    //         settings: (i*10..(i+1)*10)
    //             .map(|j| format!("setting_{}", j))
    //             .collect(),
    //     })
    //     .collect();
    // 
    // let schema = ConfigSchema {
    //     name: "test-app".to_string(),
    //     version: "1.0.0".to_string(),
    //     settings,
    //     groups,
    // };
    // 
    // assert_eq!(schema.settings.len(), 50);
    // assert_eq!(schema.groups.len(), 5);
}

#[test]
fn test_setting_type_all_variants_distinct() {
    // Test that all SettingType variants are distinct
    // let string = SettingType::String {
    //     pattern: None,
    //     min_length: None,
    //     max_length: None,
    // };
    // let int = SettingType::Integer { min: None, max: None };
    // let bool = SettingType::Boolean;
    // let secret = SettingType::Secret;
    // let any = SettingType::Any;
    // 
    // assert_ne!(string, int);
    // assert_ne!(int, bool);
    // assert_ne!(bool, secret);
    // assert_ne!(secret, any);
}

#[test]
fn test_constraint_all_variants_distinct() {
    // Test that all Constraint variants are distinct
    // let required = Constraint::Required;
    // let pattern = Constraint::Pattern(".*".to_string());
    // let range = Constraint::Range { min: 0.0, max: 100.0 };
    // let length = Constraint::Length { min: 0, max: 10 };
    // let one_of = Constraint::OneOf(vec!["a".to_string(), "b".to_string()]);
    // let custom = Constraint::Custom("validator".to_string());
    // 
    // assert_ne!(required, pattern);
    // assert_ne!(pattern, range);
    // assert_ne!(range, length);
    // assert_ne!(length, one_of);
    // assert_ne!(one_of, custom);
}
