//! Core Metadata Types Test Suite
//!
//! Tests for SettingMetadata, SettingType, Constraint, Visibility, ConfigSchema, and SettingGroup

#![cfg(feature = "metadata")]

#[cfg(feature = "metadata")]
mod core_metadata_tests {
    use settings_loader::metadata::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::Duration;

    // ============================================================================
    // VISIBILITY ENUM TESTS
    // ============================================================================

    #[test]
    fn test_visibility_public_variant() {
        let v = Visibility::Public;
        assert_eq!(v, Visibility::Public);
    }

    #[test]
    fn test_visibility_hidden_variant() {
        let v = Visibility::Hidden;
        assert_eq!(v, Visibility::Hidden);
    }

    #[test]
    fn test_visibility_secret_variant() {
        let v = Visibility::Secret;
        assert_eq!(v, Visibility::Secret);
    }

    #[test]
    fn test_visibility_advanced_variant() {
        let v = Visibility::Advanced;
        assert_eq!(v, Visibility::Advanced);
    }

    #[test]
    fn test_visibility_default_is_public() {
        let v: Visibility = Default::default();
        assert_eq!(v, Visibility::Public);
    }

    #[test]
    fn test_visibility_is_copy() {
        let v1 = Visibility::Secret;
        let v2 = v1;
        assert_eq!(v1, v2);
        assert_eq!(v1, Visibility::Secret);
    }

    #[test]
    fn test_visibility_hash() {
        let v = Visibility::Public;
        let mut hasher = DefaultHasher::new();
        v.hash(&mut hasher);
        let hash = hasher.finish();
        assert!(hash > 0);
    }

    // ============================================================================
    // CONSTRAINT ENUM TESTS
    // ============================================================================

    #[test]
    fn test_constraint_pattern_variant() {
        let c = Constraint::Pattern("[0-9]+".to_string());
        assert!(matches!(c, Constraint::Pattern(_)));
    }

    #[test]
    fn test_constraint_range_variant() {
        let c = Constraint::Range { min: 1.0, max: 100.0 };
        assert!(matches!(c, Constraint::Range { .. }));
    }

    #[test]
    fn test_constraint_length_variant() {
        let c = Constraint::Length { min: 5, max: 50 };
        assert!(matches!(c, Constraint::Length { .. }));
    }

    #[test]
    fn test_constraint_required_variant() {
        let c = Constraint::Required;
        assert_eq!(c, Constraint::Required);
    }

    #[test]
    fn test_constraint_one_of_variant() {
        let c = Constraint::OneOf(vec!["a".to_string(), "b".to_string()]);
        assert!(matches!(c, Constraint::OneOf(_)));
    }

    #[test]
    fn test_constraint_custom_variant() {
        let c = Constraint::Custom("my_validator".to_string());
        assert!(matches!(c, Constraint::Custom(_)));
    }

    #[test]
    fn test_constraint_partial_eq() {
        let c1 = Constraint::Required;
        let c2 = Constraint::Required;
        assert_eq!(c1, c2);

        let c3 = Constraint::Pattern("test".to_string());
        assert_ne!(c1, c3);
    }

    // ============================================================================
    // SETTING TYPE ENUM TESTS
    // ============================================================================

    #[test]
    fn test_setting_type_string_variant() {
        let st = SettingType::String {
            pattern: Some("[a-z]+".to_string()),
            min_length: Some(1),
            max_length: Some(255),
        };
        assert!(matches!(st, SettingType::String { .. }));
    }

    #[test]
    fn test_setting_type_string_no_constraints() {
        let st = SettingType::String { pattern: None, min_length: None, max_length: None };
        assert!(matches!(st, SettingType::String { .. }));
    }

    #[test]
    fn test_setting_type_integer_variant() {
        let st = SettingType::Integer { min: Some(0), max: Some(100) };
        assert!(matches!(st, SettingType::Integer { .. }));
    }

    #[test]
    fn test_setting_type_integer_extreme_values() {
        let st = SettingType::Integer { min: Some(i64::MIN), max: Some(i64::MAX) };
        assert!(matches!(st, SettingType::Integer { .. }));
    }

    #[test]
    fn test_setting_type_float_variant() {
        let st = SettingType::Float { min: Some(0.0), max: Some(100.5) };
        assert!(matches!(st, SettingType::Float { .. }));
    }

    #[test]
    fn test_setting_type_float_extreme_values() {
        let st = SettingType::Float {
            min: Some(f64::NEG_INFINITY),
            max: Some(f64::INFINITY),
        };
        assert!(matches!(st, SettingType::Float { .. }));
    }

    #[test]
    fn test_setting_type_boolean_variant() {
        let st = SettingType::Boolean;
        assert_eq!(st, SettingType::Boolean);
    }

    #[test]
    fn test_setting_type_duration_variant() {
        let st = SettingType::Duration {
            min: Some(Duration::from_secs(1)),
            max: Some(Duration::from_secs(3600)),
        };
        assert!(matches!(st, SettingType::Duration { .. }));
    }

    #[test]
    fn test_setting_type_duration_zero_value() {
        let st = SettingType::Duration {
            min: Some(Duration::from_secs(0)),
            max: Some(Duration::from_secs(0)),
        };
        assert!(matches!(st, SettingType::Duration { .. }));
    }

    #[test]
    fn test_setting_type_path_variant() {
        let st = SettingType::Path { must_exist: true, is_directory: false };
        assert!(matches!(st, SettingType::Path { .. }));
    }

    #[test]
    fn test_setting_type_path_all_variants() {
        let st1 = SettingType::Path { must_exist: true, is_directory: true };
        let st2 = SettingType::Path { must_exist: false, is_directory: false };
        assert!(matches!(st1, SettingType::Path { .. }));
        assert!(matches!(st2, SettingType::Path { .. }));
    }

    #[test]
    fn test_setting_type_url_variant() {
        let st = SettingType::Url {
            schemes: vec!["http".to_string(), "https".to_string()],
        };
        assert!(matches!(st, SettingType::Url { .. }));
    }

    #[test]
    fn test_setting_type_enum_variant() {
        let st = SettingType::Enum {
            variants: vec!["option1".to_string(), "option2".to_string()],
        };
        assert!(matches!(st, SettingType::Enum { .. }));
    }

    #[test]
    fn test_setting_type_array_variant() {
        let st = SettingType::Array {
            element_type: Box::new(SettingType::String { pattern: None, min_length: None, max_length: None }),
            min_items: Some(0),
            max_items: Some(100),
        };
        assert!(matches!(st, SettingType::Array { .. }));
    }

    #[test]
    fn test_setting_type_object_variant() {
        let st = SettingType::Object { fields: vec![] };
        assert!(matches!(st, SettingType::Object { .. }));
    }

    #[test]
    fn test_setting_type_secret_variant() {
        let st = SettingType::Secret;
        assert_eq!(st, SettingType::Secret);
    }

    #[test]
    fn test_setting_type_any_variant() {
        let st = SettingType::Any;
        assert_eq!(st, SettingType::Any);
    }

    // ============================================================================
    // SETTING TYPE EDGE CASES & COMPLEX SCENARIOS
    // ============================================================================

    #[test]
    fn test_visibility_all_variants_distinct() {
        let public = Visibility::Public;
        let hidden = Visibility::Hidden;
        let secret = Visibility::Secret;
        let advanced = Visibility::Advanced;

        assert_ne!(public, hidden);
        assert_ne!(public, secret);
        assert_ne!(public, advanced);
        assert_ne!(hidden, secret);
        assert_ne!(hidden, advanced);
        assert_ne!(secret, advanced);
    }

    #[test]
    fn test_constraint_pattern_empty_string() {
        let c = Constraint::Pattern(String::new());
        assert!(matches!(c, Constraint::Pattern(ref s) if s.is_empty()));
    }

    #[test]
    fn test_constraint_range_equal_bounds() {
        let c = Constraint::Range { min: 42.0, max: 42.0 };
        assert!(matches!(c, Constraint::Range { min, max } if min == max));
    }

    #[test]
    fn test_constraint_length_zero_range() {
        let c = Constraint::Length { min: 0, max: 0 };
        assert!(matches!(c, Constraint::Length { min, max } if min == 0 && max == 0));
    }

    #[test]
    fn test_setting_type_string_all_constraints() {
        let st = SettingType::String {
            pattern: Some("[0-9]+".to_string()),
            min_length: Some(1),
            max_length: Some(255),
        };

        if let SettingType::String { pattern, min_length, max_length } = st {
            assert!(pattern.is_some());
            assert!(min_length.is_some());
            assert!(max_length.is_some());
        }
    }

    #[test]
    fn test_setting_type_url_many_schemes() {
        let schemes = vec![
            "http".to_string(),
            "https".to_string(),
            "ws".to_string(),
            "wss".to_string(),
            "ftp".to_string(),
            "ftps".to_string(),
        ];
        let st = SettingType::Url { schemes };

        if let SettingType::Url { schemes: ref s } = st {
            assert_eq!(s.len(), 6);
        }
    }

    #[test]
    fn test_setting_type_enum_many_variants() {
        let variants: Vec<String> = (0..50).map(|i| format!("variant_{}", i)).collect();

        let st = SettingType::Enum { variants: variants.clone() };

        if let SettingType::Enum { variants: ref v } = st {
            assert_eq!(v.len(), 50);
        }
    }

    #[test]
    fn test_setting_type_array_deeply_nested() {
        let inner = SettingType::String { pattern: None, min_length: None, max_length: None };

        let level1 = SettingType::Array {
            element_type: Box::new(inner),
            min_items: None,
            max_items: None,
        };

        let level2 = SettingType::Array {
            element_type: Box::new(level1),
            min_items: None,
            max_items: None,
        };

        let level3 = SettingType::Array {
            element_type: Box::new(level2),
            min_items: Some(1),
            max_items: Some(100),
        };

        assert!(matches!(level3, SettingType::Array { .. }));
    }

    #[test]
    fn test_setting_type_object_many_fields() {
        let fields: Vec<SettingMetadata> = (0..20)
            .map(|i| SettingMetadata {
                key: format!("field_{}", i),
                label: format!("Field {}", i),
                description: format!("Field {}", i),
                setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                default: None,
                constraints: vec![],
                visibility: Visibility::Public,
                group: None,
            })
            .collect();

        let st = SettingType::Object { fields };

        if let SettingType::Object { fields: ref f } = st {
            assert_eq!(f.len(), 20);
        }
    }

    #[test]
    fn test_setting_metadata_all_none_optionals() {
        let metadata = SettingMetadata {
            key: "test".to_string(),
            label: "Test".to_string(),
            description: "Test".to_string(),
            setting_type: SettingType::Any,
            default: None,
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
        };

        assert!(metadata.default.is_none());
        assert!(metadata.group.is_none());
    }

    #[test]
    fn test_setting_metadata_complex_default() {
        let metadata = SettingMetadata {
            key: "complex".to_string(),
            label: "Complex".to_string(),
            description: "Complex default".to_string(),
            setting_type: SettingType::Any,
            default: None,
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
        };

        assert_eq!(metadata.default, None);
    }

    #[test]
    fn test_config_schema_many_settings_and_groups() {
        let settings: Vec<SettingMetadata> = (0..50)
            .map(|i| SettingMetadata {
                key: format!("setting_{}", i),
                label: format!("Setting {}", i),
                description: format!("Setting {}", i),
                setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                default: None,
                constraints: vec![],
                visibility: if i % 2 == 0 { Visibility::Public } else { Visibility::Hidden },
                group: Some(format!("group_{}", i % 5)),
            })
            .collect();

        let schema = ConfigSchema {
            name: "test-app".to_string(),
            version: "1.0.0".to_string(),
            settings,
            groups: vec![],
        };

        assert_eq!(schema.settings.len(), 50);
    }

    #[test]
    fn test_setting_type_all_variants_distinct() {
        let string = SettingType::String { pattern: None, min_length: None, max_length: None };
        let int = SettingType::Integer { min: None, max: None };
        let bool = SettingType::Boolean;
        let secret = SettingType::Secret;
        let any = SettingType::Any;

        assert_ne!(string, int);
        assert_ne!(int, bool);
        assert_ne!(bool, secret);
        assert_ne!(secret, any);
    }

    #[test]
    fn test_constraint_all_variants_distinct() {
        let required = Constraint::Required;
        let pattern = Constraint::Pattern(".*".to_string());
        let range = Constraint::Range { min: 0.0, max: 100.0 };
        let length = Constraint::Length { min: 0, max: 10 };
        let one_of = Constraint::OneOf(vec!["a".to_string(), "b".to_string()]);
        let custom = Constraint::Custom("validator".to_string());

        assert_ne!(required, pattern);
        assert_ne!(pattern, range);
        assert_ne!(range, length);
        assert_ne!(length, one_of);
        assert_ne!(one_of, custom);
    }
}
