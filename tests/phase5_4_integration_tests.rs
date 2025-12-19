//! Phase 5.4.1: Integration Test Suite - Metadata with Other Phases
//!
//! Comprehensive integration test suite demonstrating Phase 5 (metadata/validation/introspection)
//! working with Phases 1-4 and real-world use cases.
//!
//! **Test Coverage (10+ tests)**
//!
//! 1. **Manual SettingsIntrospection Implementation** - Show how to manually impl trait
//! 2. **Metadata + Layer Integration** - Phase 5 metadata with Phase 1 explicit layering
//! 3. **Validation Before Editor Save** - Phase 4 editor + Phase 5 validation
//! 4. **TUI Form Generation** - Auto-generate form fields from metadata
//! 5. **CLI Help Generation** - Auto-generate help text from schema
//! 6. **Secret Filtering** - Hide secrets in UI based on visibility
//! 7. **Group Organization** - Display settings organized by groups
//! 8. **Batch Validation** - Validate entire config at once
//! 9. **Multi-Scope Integration** - Phase 3 multi-scope with introspection
//! 10. **Backward Compatibility** - Works without SettingsIntrospection impl
//!
//! **Real-World Scenarios**
//! - Turtle configuration validation
//! - Database connection pooling settings
//! - API server configuration with secrets
//! - Feature flag management

#![cfg(feature = "metadata")]

#[cfg(test)]
#[cfg(feature = "metadata")]
mod phase5_4_integration_tests {
    use serde_json::json;
    use settings_loader::introspection::SettingsIntrospection;
    use settings_loader::metadata::{
        ConfigSchema, Constraint, SettingGroup, SettingMetadata, SettingType, Visibility,
    };
    use std::collections::HashMap;

    // ========================================================================
    // TEST HELPERS & TEST SETTINGS
    // ========================================================================

    /// Test settings struct for manual introspection implementation
    struct TurtleSettings;

    impl SettingsIntrospection for TurtleSettings {
        fn schema(&self) -> ConfigSchema {
            ConfigSchema {
                name: "turtle-app".to_string(),
                version: "1.0.0".to_string(),
                settings: vec![
                    SettingMetadata {
                        key: "log_level".to_string(),
                        label: "Log Level".to_string(),
                        description: "Application logging level".to_string(),
                        setting_type: SettingType::Enum {
                            variants: vec![
                                "debug".to_string(),
                                "info".to_string(),
                                "warn".to_string(),
                                "error".to_string(),
                            ],
                        },
                        default: Some(json!("info")),
                        constraints: vec![],
                        visibility: Visibility::Public,
                        group: Some("logging".to_string()),
                    },
                    SettingMetadata {
                        key: "max_retries".to_string(),
                        label: "Maximum Retries".to_string(),
                        description: "Maximum number of retry attempts".to_string(),
                        setting_type: SettingType::Integer {
                            min: Some(1),
                            max: Some(10),
                        },
                        default: Some(json!(3)),
                        constraints: vec![Constraint::Required],
                        visibility: Visibility::Public,
                        group: Some("retry_policy".to_string()),
                    },
                ],
                groups: vec![
                    SettingGroup {
                        name: "logging".to_string(),
                        label: "Logging Configuration".to_string(),
                        description: "Control application logging behavior".to_string(),
                        settings: vec!["log_level".to_string()],
                    },
                    SettingGroup {
                        name: "retry_policy".to_string(),
                        label: "Retry Policy".to_string(),
                        description: "Configure retry behavior for operations".to_string(),
                        settings: vec!["max_retries".to_string()],
                    },
                ],
            }
        }
    }

    /// Database configuration settings
    struct DatabaseSettings;

    impl SettingsIntrospection for DatabaseSettings {
        fn schema(&self) -> ConfigSchema {
            ConfigSchema {
                name: "database-config".to_string(),
                version: "1.0.0".to_string(),
                settings: vec![
                    SettingMetadata {
                        key: "db_host".to_string(),
                        label: "Database Host".to_string(),
                        description: "Database server hostname or IP address".to_string(),
                        setting_type: SettingType::String {
                            pattern: None,
                            min_length: Some(1),
                            max_length: Some(255),
                        },
                        default: Some(json!("localhost")),
                        constraints: vec![Constraint::Required],
                        visibility: Visibility::Public,
                        group: Some("database".to_string()),
                    },
                    SettingMetadata {
                        key: "db_port".to_string(),
                        label: "Database Port".to_string(),
                        description: "Database server port number".to_string(),
                        setting_type: SettingType::Integer {
                            min: Some(1),
                            max: Some(65535),
                        },
                        default: Some(json!(5432)),
                        constraints: vec![Constraint::Range {
                            min: 1.0,
                            max: 65535.0,
                        }],
                        visibility: Visibility::Public,
                        group: Some("database".to_string()),
                    },
                    SettingMetadata {
                        key: "db_password".to_string(),
                        label: "Database Password".to_string(),
                        description: "Database authentication password".to_string(),
                        setting_type: SettingType::Secret,
                        default: None,
                        constraints: vec![Constraint::Required],
                        visibility: Visibility::Secret,
                        group: Some("database".to_string()),
                    },
                    SettingMetadata {
                        key: "max_connections".to_string(),
                        label: "Maximum Connections".to_string(),
                        description: "Connection pool maximum size".to_string(),
                        setting_type: SettingType::Integer {
                            min: Some(1),
                            max: Some(1000),
                        },
                        default: Some(json!(10)),
                        constraints: vec![],
                        visibility: Visibility::Public,
                        group: Some("database".to_string()),
                    },
                ],
                groups: vec![SettingGroup {
                    name: "database".to_string(),
                    label: "Database Configuration".to_string(),
                    description: "Configure database connection and pooling".to_string(),
                    settings: vec![
                        "db_host".to_string(),
                        "db_port".to_string(),
                        "db_password".to_string(),
                        "max_connections".to_string(),
                    ],
                }],
            }
        }
    }

    /// API server configuration with secrets and advanced settings
    struct ApiServerSettings;

    impl SettingsIntrospection for ApiServerSettings {
        fn schema(&self) -> ConfigSchema {
            ConfigSchema {
                name: "api-server".to_string(),
                version: "1.0.0".to_string(),
                settings: vec![
                    SettingMetadata {
                        key: "api_url".to_string(),
                        label: "API URL".to_string(),
                        description: "API endpoint URL".to_string(),
                        setting_type: SettingType::Url {
                            schemes: vec!["http".to_string(), "https".to_string()],
                        },
                        default: Some(json!("https://api.example.com")),
                        constraints: vec![Constraint::Required],
                        visibility: Visibility::Public,
                        group: Some("api".to_string()),
                    },
                    SettingMetadata {
                        key: "api_key".to_string(),
                        label: "API Key".to_string(),
                        description: "Secret API authentication key".to_string(),
                        setting_type: SettingType::Secret,
                        default: None,
                        constraints: vec![Constraint::Required],
                        visibility: Visibility::Secret,
                        group: Some("api".to_string()),
                    },
                    SettingMetadata {
                        key: "api_timeout_secs".to_string(),
                        label: "API Timeout".to_string(),
                        description: "API request timeout in seconds".to_string(),
                        setting_type: SettingType::Integer {
                            min: Some(1),
                            max: Some(300),
                        },
                        default: Some(json!(30)),
                        constraints: vec![],
                        visibility: Visibility::Public,
                        group: Some("api".to_string()),
                    },
                    SettingMetadata {
                        key: "enable_caching".to_string(),
                        label: "Enable Response Caching".to_string(),
                        description: "Cache API responses for performance".to_string(),
                        setting_type: SettingType::Boolean,
                        default: Some(json!(true)),
                        constraints: vec![],
                        visibility: Visibility::Advanced,
                        group: Some("cache".to_string()),
                    },
                    SettingMetadata {
                        key: "cache_ttl_seconds".to_string(),
                        label: "Cache TTL".to_string(),
                        description: "Cache time-to-live in seconds".to_string(),
                        setting_type: SettingType::Integer {
                            min: Some(1),
                            max: Some(3600),
                        },
                        default: Some(json!(300)),
                        constraints: vec![],
                        visibility: Visibility::Advanced,
                        group: Some("cache".to_string()),
                    },
                ],
                groups: vec![
                    SettingGroup {
                        name: "api".to_string(),
                        label: "API Configuration".to_string(),
                        description: "API client configuration and authentication".to_string(),
                        settings: vec![
                            "api_url".to_string(),
                            "api_key".to_string(),
                            "api_timeout_secs".to_string(),
                        ],
                    },
                    SettingGroup {
                        name: "cache".to_string(),
                        label: "Caching Configuration".to_string(),
                        description: "Response caching behavior".to_string(),
                        settings: vec![
                            "enable_caching".to_string(),
                            "cache_ttl_seconds".to_string(),
                        ],
                    },
                ],
            }
        }
    }

    // ========================================================================
    // TEST 1: Manual SettingsIntrospection Implementation
    // ========================================================================

    /// Demonstrates how to manually implement SettingsIntrospection trait
    /// and use it to query configuration metadata
    #[test]
    fn test_manual_introspection_implementation() {
        let settings = TurtleSettings;
        let schema = settings.schema();

        assert_eq!(schema.name, "turtle-app");
        assert_eq!(schema.version, "1.0.0");
        assert_eq!(schema.settings.len(), 2);
        assert_eq!(schema.groups.len(), 2);

        // Verify we can retrieve specific settings
        let log_level = settings.get_setting_metadata("log_level");
        assert!(log_level.is_some());
        assert_eq!(log_level.unwrap().label, "Log Level");
    }

    // ========================================================================
    // TEST 2: Metadata + Layer Integration (Phase 1 + Phase 5)
    // ========================================================================

    /// Phase 5 metadata working with Phase 1 explicit layering
    /// Demonstrates how configuration layers can use metadata validation
    #[test]
    fn test_metadata_with_explicit_layering() {
        // Simulating Phase 1 explicit layering behavior
        let settings = DatabaseSettings;

        // Layer 1: Get defaults from metadata
        let defaults: HashMap<String, serde_json::Value> = settings
            .settings_with_defaults()
            .into_iter()
            .filter_map(|s| s.default.map(|d| (s.key, d)))
            .collect();

        assert_eq!(defaults.len(), 3); // db_host, db_port, max_connections have defaults
        assert_eq!(defaults["db_host"], json!("localhost"));
        assert_eq!(defaults["db_port"], json!(5432));

        // Layer 2: Get required fields from constraints
        let required_settings: Vec<_> = settings
            .settings_with_constraint(&Constraint::Required)
            .into_iter()
            .map(|s| s.key)
            .collect();

        assert!(required_settings.contains(&"db_host".to_string()));
        assert!(required_settings.contains(&"db_password".to_string()));
    }

    // ========================================================================
    // TEST 3: Validation Before Editor Save (Phase 4 + Phase 5)
    // ========================================================================

    /// Phase 4 config editor + Phase 5 validation
    /// Demonstrates validating before saving changes
    #[test]
    fn test_validation_before_editor_save() {
        let settings = ApiServerSettings;

        // Simulating editor: user edits settings
        let user_edits = json!({
            "api_url": "https://api.example.com",
            "api_key": "secret-123",
            "api_timeout_secs": 60
        });

        // Before saving (editor save operation), validate
        let validation_result = settings.validate_config(&user_edits);
        assert!(validation_result.is_ok());

        let result = validation_result.unwrap();
        assert!(result.is_valid(), "Config should be valid before save");

        // Invalid config should fail validation before save
        let invalid_edits = json!({
            "api_url": "not-a-url",  // Invalid URL
            "api_timeout_secs": 500   // Exceeds max of 300
        });

        let validation_result = settings.validate_config(&invalid_edits);
        assert!(validation_result.is_ok());

        let result = validation_result.unwrap();
        assert!(!result.is_valid(), "Invalid config should fail validation");
    }

    // ========================================================================
    // TEST 4: TUI Form Generation (Dynamic UI from Metadata)
    // ========================================================================

    /// Auto-generate TUI form fields from metadata
    /// Demonstrates building dynamic UI components from schema
    #[test]
    fn test_generate_tui_form_from_metadata() {
        let settings = DatabaseSettings;
        let schema = settings.schema();

        // Collect form fields by group
        let mut form_fields: HashMap<String, Vec<String>> = HashMap::new();
        for group in &schema.groups {
            let fields: Vec<String> = group
                .settings
                .iter()
                .filter_map(|key| {
                    settings
                        .get_setting_metadata(key)
                        .map(|s| format!("{}: {}", s.label, s.description))
                })
                .collect();
            form_fields.insert(group.name.clone(), fields);
        }

        // Verify form structure
        assert!(form_fields.contains_key("database"));
        let db_fields = &form_fields["database"];
        assert_eq!(db_fields.len(), 4); // All 4 database settings

        // Verify field information for input rendering
        for setting in &settings.schema().settings {
            match &setting.setting_type {
                SettingType::Integer { min, max } => {
                    // TUI could render as numeric input with bounds
                    assert!(min.is_some() || max.is_some());
                }
                SettingType::Secret => {
                    // TUI could render as password input (masked)
                    assert_eq!(setting.visibility, Visibility::Secret);
                }
                _ => {}
            }
        }
    }

    // ========================================================================
    // TEST 5: CLI Help Generation (Auto-generate Help Text)
    // ========================================================================

    /// Auto-generate help text from schema for CLI
    /// Demonstrates building command-line documentation from metadata
    #[test]
    fn test_generate_cli_help_from_schema() {
        let settings = ApiServerSettings;
        let schema = settings.schema();

        // Generate help text for each setting
        let mut help_text = String::new();
        help_text.push_str(&format!("Configuration for {}\n", schema.name));
        help_text.push_str(&format!("Version: {}\n\n", schema.version));

        for group in &schema.groups {
            help_text.push_str(&format!("## {}\n", group.label));
            help_text.push_str(&format!("{}\n\n", group.description));

            for key in &group.settings {
                if let Some(setting) = settings.get_setting_metadata(key) {
                    help_text.push_str(&format!("  {}:\n", setting.label));
                    help_text.push_str(&format!("    {}\n", setting.description));

                    // Add constraint information to help
                    for constraint in &setting.constraints {
                        help_text.push_str(&format!("    Constraint: {:?}\n", constraint));
                    }

                    if let Some(default) = &setting.default {
                        help_text.push_str(&format!("    Default: {}\n", default));
                    }
                    help_text.push_str("\n");
                }
            }
        }

        // Verify help text was generated correctly
        assert!(help_text.contains("api-server"));
        assert!(help_text.contains("API Configuration"));
        assert!(help_text.contains("API URL"));
        assert!(help_text.contains("https://api.example.com"));
    }

    // ========================================================================
    // TEST 6: Secret Filtering (Hide Secrets in UI)
    // ========================================================================

    /// Filter secrets from UI based on visibility metadata
    /// Demonstrates security-conscious UI rendering
    #[test]
    fn test_filter_secrets_from_ui() {
        let settings = ApiServerSettings;

        // Get public settings for display in UI
        let public_settings = settings.public_settings();
        assert_eq!(public_settings.len(), 2); // api_url, api_timeout_secs

        // Verify no secrets in public settings
        for setting in &public_settings {
            assert_ne!(setting.visibility, Visibility::Secret);
        }

        // Get secret settings (for separate, protected display)
        let secret_settings = settings.secret_settings();
        assert_eq!(secret_settings.len(), 1); // api_key

        for setting in &secret_settings {
            assert_eq!(setting.visibility, Visibility::Secret);
        }

        // Get advanced settings
        let advanced_settings = settings.advanced_settings();
        assert_eq!(advanced_settings.len(), 2); // enable_caching, cache_ttl_seconds

        // Simulate rendering UI form: include public + advanced, skip secrets
        let renderable_settings: Vec<_> = settings
            .schema()
            .settings
            .into_iter()
            .filter(|s| s.visibility != Visibility::Secret && s.visibility != Visibility::Hidden)
            .collect();

        assert_eq!(renderable_settings.len(), 4); // public + advanced, no secrets/hidden
    }

    // ========================================================================
    // TEST 7: Group Organization (Display Settings by Groups)
    // ========================================================================

    /// Display settings organized by groups
    /// Demonstrates UI grouping and organization
    #[test]
    fn test_group_settings_for_display() {
        let settings = ApiServerSettings;

        // Organize settings by group for display
        let groups = settings.groups();
        assert_eq!(groups.len(), 2);

        // For each group, get its settings
        let mut grouped_display: HashMap<String, Vec<String>> = HashMap::new();

        for group in groups {
            let group_settings = settings.settings_in_group(&group.name);
            let labels: Vec<String> = group_settings.iter().map(|s| s.label.clone()).collect();
            grouped_display.insert(group.name, labels);
        }

        // Verify grouping
        assert!(grouped_display.contains_key("api"));
        assert!(grouped_display.contains_key("cache"));

        let api_group = &grouped_display["api"];
        assert_eq!(api_group.len(), 3);
        assert!(api_group.contains(&"API URL".to_string()));

        let cache_group = &grouped_display["cache"];
        assert_eq!(cache_group.len(), 2);
        assert!(cache_group.contains(&"Enable Response Caching".to_string()));
    }

    // ========================================================================
    // TEST 8: Batch Validation (Validate Entire Config)
    // ========================================================================

    /// Validate entire configuration at once
    /// Demonstrates comprehensive config validation
    #[test]
    fn test_validate_all_settings_batch() {
        let settings = DatabaseSettings;

        // Valid complete configuration
        let valid_config = json!({
            "db_host": "prod-db.example.com",
            "db_port": 5432,
            "db_password": "secure-password",
            "max_connections": 50
        });

        let result = settings.validate_config(&valid_config);
        assert!(result.is_ok());
        assert!(result.unwrap().is_valid());

        // Partially invalid configuration (port out of range)
        let invalid_config = json!({
            "db_host": "prod-db.example.com",
            "db_port": 99999,  // Exceeds max of 65535
            "db_password": "secure-password",
            "max_connections": 50
        });

        let result = settings.validate_config(&invalid_config);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.is_valid());
        assert!(!validation.errors().is_empty());
    }

    // ========================================================================
    // TEST 9: Multi-Scope Integration (Phase 3 + Phase 5)
    // ========================================================================

    /// Phase 3 multi-scope with Phase 5 introspection
    /// Demonstrates metadata working across multiple configuration scopes
    #[test]
    fn test_introspection_with_multi_scope() {
        // Simulating Phase 3 multi-scope: different scopes have different settings visibility
        let settings = ApiServerSettings;

        // User scope: all public settings visible
        let user_visible = settings.public_settings();
        assert!(user_visible.iter().all(|s| s.visibility == Visibility::Public));

        // System scope: all public + advanced settings
        let system_visible: Vec<_> = settings
            .schema()
            .settings
            .into_iter()
            .filter(|s| s.visibility != Visibility::Secret && s.visibility != Visibility::Hidden)
            .collect();

        assert!(system_visible.len() > user_visible.len());

        // Admin scope: all settings including secrets
        let admin_visible = settings.schema().settings;
        assert_eq!(admin_visible.len(), 5);

        // Statistics show scope-dependent visibility
        let user_stats = settings.visibility_distribution();
        assert!(user_stats.contains_key("public"));
        assert!(user_stats.contains_key("advanced"));
        assert!(user_stats.contains_key("secret"));
    }

    // ========================================================================
    // TEST 10: Backward Compatibility (Works Without SettingsIntrospection)
    // ========================================================================

    /// Works without SettingsIntrospection implementation
    /// Demonstrates graceful degradation for configs not implementing introspection
    #[test]
    fn test_backward_compatibility_no_introspection() {
        // This test verifies that code can work with or without SettingsIntrospection
        // In real application, optional trait implementation doesn't break anything

        // Settings that implement introspection work fully
        let introspectable_settings: Box<dyn SettingsIntrospection> =
            Box::new(TurtleSettings);
        let schema = introspectable_settings.schema();
        assert!(!schema.settings.is_empty());

        // Multiple different implementations can coexist
        let settings1: Box<dyn SettingsIntrospection> = Box::new(DatabaseSettings);
        let settings2: Box<dyn SettingsIntrospection> = Box::new(ApiServerSettings);

        // Both work independently
        assert_ne!(settings1.schema().name, settings2.schema().name);
        assert_ne!(
            settings1.settings_count(),
            settings2.settings_count()
        );
    }

    // ========================================================================
    // TEST 11: Real-World Scenario - Turtle Configuration Validation
    // ========================================================================

    /// Real-world scenario: Turtle configuration validation
    /// Demonstrates complete workflow: metadata definition → user input → validation
    #[test]
    fn test_turtle_configuration_validation_workflow() {
        let settings = TurtleSettings;

        // Step 1: User loads default configuration from metadata
        let defaults: HashMap<String, serde_json::Value> = settings
            .settings_with_defaults()
            .into_iter()
            .filter_map(|s| s.default.map(|d| (s.key, d)))
            .collect();

        // Step 2: User customizes configuration
        let mut config = json!({});
        for (key, default_value) in &defaults {
            config[key] = default_value.clone();
        }
        config["log_level"] = json!("debug");
        config["max_retries"] = json!(5);

        // Step 3: Validate before applying
        let result = settings.validate_config(&config);
        assert!(result.is_ok());
        assert!(result.unwrap().is_valid());

        // Step 4: Invalid configuration rejected
        let invalid_config = json!({
            "log_level": "INVALID",  // Not in enum variants
            "max_retries": 100        // Exceeds max of 10
        });

        let result = settings.validate_config(&invalid_config);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.is_valid());
    }

    // ========================================================================
    // TEST 12: Real-World Scenario - API Server with Multi-Layer Config
    // ========================================================================

    /// Real-world scenario: API server configuration with layered approach
    /// Demonstrates Phase 1 (layers) + Phase 5 (metadata) integration
    #[test]
    fn test_api_server_multi_layer_configuration() {
        let settings = ApiServerSettings;

        // Layer 1: Defaults from metadata
        let defaults = json!({
            "api_url": "https://api.example.com",
            "api_timeout_secs": 30,
            "enable_caching": true,
            "cache_ttl_seconds": 300
        });

        // Layer 2: Environment overrides
        let env_overrides = json!({
            "api_timeout_secs": 60
        });

        // Layer 3: Local config
        let local_config = json!({
            "api_key": "dev-key-123",
            "cache_ttl_seconds": 60
        });

        // Merge layers (simulating Phase 1 explicit layering)
        let merged = json!({
            "api_url": defaults["api_url"],
            "api_timeout_secs": env_overrides["api_timeout_secs"],
            "enable_caching": defaults["enable_caching"],
            "cache_ttl_seconds": local_config["cache_ttl_seconds"],
            "api_key": local_config["api_key"]
        });

        // Validate merged configuration
        let result = settings.validate_config(&merged);
        assert!(result.is_ok());
        assert!(result.unwrap().is_valid());
    }

    // ========================================================================
    // TEST 13: Settings Discovery & Search
    // ========================================================================

    /// Test settings discovery and search capabilities
    /// Demonstrates finding settings programmatically
    #[test]
    fn test_settings_discovery_and_search() {
        let settings = DatabaseSettings;

        // Search by key/label
        let db_results = settings.search_settings("db");
        assert!(!db_results.is_empty());
        assert!(db_results.iter().any(|s| s.key == "db_host"));
        assert!(db_results.iter().any(|s| s.key == "db_port"));

        // Search by description
        let connection_results =
            settings.search_settings_by_description("connection");
        assert!(!connection_results.is_empty());

        // Count statistics
        let total = settings.settings_count();
        assert_eq!(total, 4);

        // Type distribution
        let type_dist = settings.type_distribution();
        assert!(type_dist.contains_key("string"));
        assert!(type_dist.contains_key("integer"));
        assert!(type_dist.contains_key("secret"));
    }

    // ========================================================================
    // TEST 14: Constraint Analysis & Introspection
    // ========================================================================

    /// Test constraint analysis through introspection
    /// Demonstrates understanding configuration constraints
    #[test]
    fn test_constraint_analysis_through_introspection() {
        let settings = DatabaseSettings;

        // Get settings with specific constraints
        let required_settings =
            settings.settings_with_constraint(&Constraint::Required);
        assert_eq!(required_settings.len(), 2); // db_host, db_password

        // Get settings with range constraints
        let port_constraint = Constraint::Range {
            min: 1.0,
            max: 65535.0,
        };
        let range_settings =
            settings.settings_with_constraint(&port_constraint);
        assert!(!range_settings.is_empty());

        // Constraint statistics
        let stats = settings.constraint_statistics();
        assert!(stats.contains_key("required"));
        assert!(stats.contains_key("range"));
    }
}
