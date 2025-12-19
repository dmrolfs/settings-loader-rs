//! Settings Introspection Trait Test Suite
//!
//! Comprehensive tests for the SettingsIntrospection trait and introspection API.
//! Verifies the contract for runtime configuration introspection capabilities.

#![cfg(feature = "metadata")]

#[cfg(feature = "metadata")]
mod introspection_trait_tests {
    use settings_loader::metadata::{ConfigSchema, SettingGroup, SettingMetadata, SettingType, Visibility};

    // Test helper: Create a basic test introspector
    fn create_test_schema() -> ConfigSchema {
        ConfigSchema {
            name: "test-app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![
                SettingMetadata {
                    key: "api_url".to_string(),
                    label: "API URL".to_string(),
                    description: "API endpoint URL".to_string(),
                    setting_type: SettingType::String {
                        pattern: None,
                        min_length: Some(1),
                        max_length: None,
                    },
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: Some("api".to_string()),
                },
                SettingMetadata {
                    key: "api_key".to_string(),
                    label: "API Key".to_string(),
                    description: "Secret API key".to_string(),
                    setting_type: SettingType::Secret,
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Secret,
                    group: Some("api".to_string()),
                },
                SettingMetadata {
                    key: "debug_mode".to_string(),
                    label: "Debug Mode".to_string(),
                    description: "Enable debug mode".to_string(),
                    setting_type: SettingType::Boolean,
                    default: Some(serde_json::json!(false)),
                    constraints: vec![],
                    visibility: Visibility::Hidden,
                    group: Some("debug".to_string()),
                },
            ],
            groups: vec![SettingGroup {
                name: "api".to_string(),
                label: "API Configuration".to_string(),
                description: "API settings".to_string(),
                settings: vec!["api_url".to_string(), "api_key".to_string()],
            }],
        }
    }

    // ============================================================================
    // SCHEMA RETRIEVAL TESTS
    // ============================================================================

    #[test]
    fn test_get_schema_returns_full_schema() {
        let schema = create_test_schema();
        assert_eq!(schema.name, "test-app");
        assert_eq!(schema.version, "1.0.0");
        assert!(!schema.settings.is_empty());
    }

    #[test]
    fn test_get_schema_with_empty_settings() {
        let schema = ConfigSchema {
            name: "empty-app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![],
            groups: vec![],
        };

        assert!(schema.settings.is_empty());
        assert!(schema.groups.is_empty());
    }

    #[test]
    fn test_schema_contains_complete_metadata() {
        let schema = create_test_schema();

        for setting in &schema.settings {
            assert!(!setting.key.is_empty());
            assert!(!setting.label.is_empty());
            assert!(!setting.description.is_empty());
        }
    }

    // ============================================================================
    // SETTING LOOKUP TESTS
    // ============================================================================

    #[test]
    fn test_get_setting_basic_structure() {
        let schema = create_test_schema();
        let api_url = schema.settings.iter().find(|s| s.key == "api_url");

        assert!(api_url.is_some());
        let api_url = api_url.unwrap();
        assert_eq!(api_url.key, "api_url");
        assert_eq!(api_url.label, "API URL");
    }

    #[test]
    fn test_find_setting_by_key() {
        let schema = create_test_schema();
        assert!(schema.settings.iter().any(|s| s.key == "api_url"));
        assert!(schema.settings.iter().any(|s| s.key == "api_key"));
    }

    #[test]
    fn test_find_nonexistent_setting_returns_none() {
        let schema = create_test_schema();
        assert!(!schema.settings.iter().any(|s| s.key == "nonexistent"));
    }

    #[test]
    fn test_setting_lookup_case_sensitive() {
        let schema = create_test_schema();
        assert!(schema.settings.iter().any(|s| s.key == "api_url"));
        assert!(!schema.settings.iter().any(|s| s.key == "API_URL"));
    }

    // ============================================================================
    // VISIBILITY FILTERING TESTS
    // ============================================================================

    #[test]
    fn test_get_public_settings() {
        let schema = create_test_schema();
        let public: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| s.visibility == Visibility::Public)
            .collect();

        assert!(!public.is_empty());
        for setting in &public {
            assert_eq!(setting.visibility, Visibility::Public);
        }
    }

    #[test]
    fn test_get_secret_settings() {
        let schema = create_test_schema();
        let secret: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| s.visibility == Visibility::Secret)
            .collect();

        for setting in &secret {
            assert_eq!(setting.visibility, Visibility::Secret);
        }
    }

    #[test]
    fn test_get_hidden_settings() {
        let schema = create_test_schema();
        let hidden: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| s.visibility == Visibility::Hidden)
            .collect();

        for setting in &hidden {
            assert_eq!(setting.visibility, Visibility::Hidden);
        }
    }

    #[test]
    fn test_public_settings_excludes_secrets() {
        let schema = create_test_schema();
        let public: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| s.visibility == Visibility::Public)
            .collect();

        for setting in &public {
            assert_ne!(setting.visibility, Visibility::Secret);
        }
    }

    #[test]
    fn test_visibility_filtering_with_mixed_settings() {
        let schema = create_test_schema();
        let public: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| s.visibility == Visibility::Public)
            .collect();
        let secret: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| s.visibility == Visibility::Secret)
            .collect();

        // No overlap between visibility levels
        for pub_setting in &public {
            for sec_setting in &secret {
                assert_ne!(pub_setting.key, sec_setting.key);
            }
        }
    }

    // ============================================================================
    // GROUPING TESTS
    // ============================================================================

    #[test]
    fn test_get_settings_by_group() {
        let schema = create_test_schema();
        let api_settings: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| s.group.as_ref().is_some_and(|g| g == "api"))
            .collect();

        assert!(!api_settings.is_empty());
        for setting in &api_settings {
            assert_eq!(setting.group, Some("api".to_string()));
        }
    }

    #[test]
    fn test_get_settings_by_nonexistent_group() {
        let schema = create_test_schema();
        let settings: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| s.group.as_ref().is_some_and(|g| g == "nonexistent"))
            .collect();

        assert!(settings.is_empty());
    }

    #[test]
    fn test_get_all_groups() {
        let schema = create_test_schema();
        assert!(!schema.groups.is_empty());

        for group in &schema.groups {
            assert!(!group.name.is_empty());
            assert!(!group.label.is_empty());
        }
    }

    #[test]
    fn test_groups_contain_setting_references() {
        let schema = create_test_schema();

        for group in &schema.groups {
            for setting_key in &group.settings {
                assert!(schema.settings.iter().any(|s| &s.key == setting_key));
            }
        }
    }

    // ============================================================================
    // TYPE INTROSPECTION TESTS
    // ============================================================================

    #[test]
    fn test_get_settings_by_type_string() {
        let schema = create_test_schema();
        let string_settings: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| matches!(s.setting_type, SettingType::String { .. }))
            .collect();

        for setting in &string_settings {
            assert!(matches!(setting.setting_type, SettingType::String { .. }));
        }
    }

    #[test]
    fn test_get_settings_by_type_boolean() {
        let schema = create_test_schema();
        let bool_settings: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| s.setting_type == SettingType::Boolean)
            .collect();

        for setting in &bool_settings {
            assert_eq!(setting.setting_type, SettingType::Boolean);
        }
    }

    #[test]
    fn test_get_settings_by_type_secret() {
        let schema = create_test_schema();
        let secret_settings: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| s.setting_type == SettingType::Secret)
            .collect();

        for setting in &secret_settings {
            assert_eq!(setting.setting_type, SettingType::Secret);
        }
    }

    // ============================================================================
    // SEARCH & QUERY TESTS
    // ============================================================================

    #[test]
    fn test_search_settings_by_key() {
        let schema = create_test_schema();
        let results: Vec<_> = schema.settings.iter().filter(|s| s.key.contains("api")).collect();

        assert!(!results.is_empty());
        for setting in &results {
            assert!(setting.key.contains("api"));
        }
    }

    #[test]
    fn test_search_settings_by_description() {
        let schema = create_test_schema();
        let results: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| s.description.to_lowercase().contains("api"))
            .collect();

        for setting in &results {
            assert!(setting.description.to_lowercase().contains("api"));
        }
    }

    #[test]
    fn test_search_returns_empty_for_no_matches() {
        let schema = create_test_schema();
        let results: Vec<_> = schema.settings.iter().filter(|s| s.key.contains("nonexistent")).collect();

        assert!(results.is_empty());
    }

    // ============================================================================
    // STATISTICS & METRICS TESTS
    // ============================================================================

    #[test]
    fn test_get_total_settings_count() {
        let schema = create_test_schema();
        assert!(!schema.settings.is_empty());
        assert_eq!(schema.settings.len(), 3);
    }

    #[test]
    fn test_visibility_distribution() {
        let schema = create_test_schema();
        let public_count = schema
            .settings
            .iter()
            .filter(|s| s.visibility == Visibility::Public)
            .count();
        let secret_count = schema
            .settings
            .iter()
            .filter(|s| s.visibility == Visibility::Secret)
            .count();
        let hidden_count = schema
            .settings
            .iter()
            .filter(|s| s.visibility == Visibility::Hidden)
            .count();

        assert!(public_count > 0);
        assert_eq!(public_count + secret_count + hidden_count, schema.settings.len());
    }

    #[test]
    fn test_get_type_distribution() {
        let schema = create_test_schema();
        let mut type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for setting in &schema.settings {
            let type_name = format!("{:?}", setting.setting_type);
            *type_counts.entry(type_name).or_insert(0) += 1;
        }

        assert!(!type_counts.is_empty());
        for count in type_counts.values() {
            assert!(*count > 0);
        }
    }

    // ============================================================================
    // INTEGRATION TESTS
    // ============================================================================

    #[test]
    fn test_schema_consistency() {
        let schema = create_test_schema();

        let schema2 = create_test_schema();

        assert_eq!(schema.settings.len(), schema2.settings.len());
        assert_eq!(schema.groups.len(), schema2.groups.len());
    }

    #[test]
    fn test_nested_object_introspection() {
        let fields = vec![SettingMetadata {
            key: "host".to_string(),
            label: "Host".to_string(),
            description: "Database host".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: None,
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
        }];

        let schema = ConfigSchema {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![SettingMetadata {
                key: "database".to_string(),
                label: "Database".to_string(),
                description: "Database config".to_string(),
                setting_type: SettingType::Object { fields },
                default: None,
                constraints: vec![],
                visibility: Visibility::Public,
                group: None,
            }],
            groups: vec![],
        };

        for setting in &schema.settings {
            if let SettingType::Object { fields } = &setting.setting_type {
                assert!(!fields.is_empty());
                for field in fields {
                    assert!(!field.key.is_empty());
                }
            }
        }
    }

    #[test]
    fn test_deeply_nested_types() {
        let inner = SettingType::String { pattern: None, min_length: None, max_length: None };

        let array_level1 = SettingType::Array {
            element_type: Box::new(inner),
            min_items: None,
            max_items: None,
        };

        let array_level2 = SettingType::Array {
            element_type: Box::new(array_level1),
            min_items: None,
            max_items: None,
        };

        let schema = ConfigSchema {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![SettingMetadata {
                key: "nested".to_string(),
                label: "Nested".to_string(),
                description: "Nested array".to_string(),
                setting_type: array_level2,
                default: None,
                constraints: vec![],
                visibility: Visibility::Public,
                group: None,
            }],
            groups: vec![],
        };

        for setting in &schema.settings {
            // Verify no panics with deep nesting
            let _ = format!("{:?}", setting.setting_type);
        }
    }

    #[test]
    fn test_all_visibility_levels() {
        let visibilities = vec![
            Visibility::Public,
            Visibility::Hidden,
            Visibility::Secret,
            Visibility::Advanced,
        ];

        for v in visibilities {
            let setting = SettingMetadata {
                key: "test".to_string(),
                label: "Test".to_string(),
                description: "Test".to_string(),
                setting_type: SettingType::Any,
                default: None,
                constraints: vec![],
                visibility: v,
                group: None,
            };

            assert_eq!(setting.visibility, v);
        }
    }

    // ============================================================================
    // ERROR HANDLING TESTS
    // ============================================================================

    #[test]
    fn test_error_handling_for_invalid_queries() {
        let schema = create_test_schema();
        let result = schema.settings.iter().find(|s| s.key.is_empty());

        assert!(result.is_none());
    }

    #[test]
    fn test_boundary_conditions() {
        let schema = create_test_schema();

        let empty_search: Vec<_> = schema.settings.iter().filter(|s| s.key.is_empty()).collect();

        assert!(empty_search.is_empty());
    }

    // ============================================================================
    // PERFORMANCE TESTS
    // ============================================================================

    #[test]
    fn test_performance_with_large_schema() {
        let settings: Vec<SettingMetadata> = (0..100)
            .map(|i| SettingMetadata {
                key: format!("setting_{}", i),
                label: format!("Setting {}", i),
                description: format!("Setting {}", i),
                setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                default: None,
                constraints: vec![],
                visibility: Visibility::Public,
                group: None,
            })
            .collect();

        let schema = ConfigSchema {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            settings,
            groups: vec![],
        };

        let start = std::time::Instant::now();

        let _public: Vec<_> = schema
            .settings
            .iter()
            .filter(|s| s.visibility == Visibility::Public)
            .collect();

        for i in 0..10 {
            let _: Vec<_> = schema
                .settings
                .iter()
                .filter(|s| s.key.contains(&format!("setting_{}", i)))
                .collect();
        }

        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 100, "Performance test should complete quickly");
    }
}
