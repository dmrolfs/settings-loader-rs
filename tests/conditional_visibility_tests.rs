//! Comprehensive Test Suite for ConditionalVisibility
//!
//! Tests for the ConditionalVisibility struct and pattern matching algorithm.
//! Ensures that conditional metadata-driven visibility works correctly for
//! dynamic settings that depend on other settings' values.

#![cfg(feature = "metadata")]

#[cfg(feature = "metadata")]
mod conditional_visibility_tests {
    use serde_json::json;
    use settings_loader::metadata::{ConditionalVisibility, Constraint, SettingMetadata, SettingType, Visibility};

    // ============================================================================
    // BASIC PATTERN MATCHING TESTS
    // ============================================================================

    /// Test that ConditionalVisibility correctly matches wildcard patterns
    #[test]
    fn test_wildcard_pattern_matching_basic() {
        let rule = ConditionalVisibility {
            base_setting: "llm.provider".to_string(),
            depends_on_value: "ollama".to_string(),
            applies_to_pattern: "llm.ollama.*".to_string(),
        };

        // Should match settings under llm.ollama.
        assert!(rule.matches_pattern("llm.ollama.base_url"));
        assert!(rule.matches_pattern("llm.ollama.model"));
        assert!(rule.matches_pattern("llm.ollama.temperature"));
        assert!(rule.matches_pattern("llm.ollama.max_tokens"));
    }

    /// Test that wildcard patterns don't match different providers
    #[test]
    fn test_wildcard_pattern_rejects_different_provider() {
        let rule = ConditionalVisibility {
            base_setting: "llm.provider".to_string(),
            depends_on_value: "ollama".to_string(),
            applies_to_pattern: "llm.ollama.*".to_string(),
        };

        // Should NOT match different providers
        assert!(!rule.matches_pattern("llm.openai.api_key"));
        assert!(!rule.matches_pattern("llm.anthropic.api_key"));
        assert!(!rule.matches_pattern("llm.copilot.token"));
    }

    /// Test that prefix alone doesn't match wildcard pattern
    #[test]
    fn test_wildcard_pattern_requires_dot_separator() {
        let rule = ConditionalVisibility {
            base_setting: "llm.provider".to_string(),
            depends_on_value: "ollama".to_string(),
            applies_to_pattern: "llm.ollama.*".to_string(),
        };

        // Prefix alone should NOT match (must have something after dot)
        assert!(!rule.matches_pattern("llm.ollama"));

        // Prefix with dot and value should match
        assert!(rule.matches_pattern("llm.ollama."));
        assert!(rule.matches_pattern("llm.ollama.x"));
        assert!(rule.matches_pattern("llm.ollama.base_url"));
    }

    /// Test that similar but different prefixes don't match
    #[test]
    fn test_wildcard_pattern_prevents_false_matches() {
        let rule = ConditionalVisibility {
            base_setting: "llm.provider".to_string(),
            depends_on_value: "ollama".to_string(),
            applies_to_pattern: "llm.ollama.*".to_string(),
        };

        // Should NOT match if prefix is extended
        assert!(!rule.matches_pattern("llm.ollamaabc.model"));
        assert!(!rule.matches_pattern("llm.ollama_extended.value"));

        // The dot-separator requirement prevents these false matches
    }

    // ============================================================================
    // EXACT PATTERN MATCHING TESTS
    // ============================================================================

    /// Test that exact patterns (no wildcard) match only exact keys
    #[test]
    fn test_exact_pattern_matching() {
        let rule = ConditionalVisibility {
            base_setting: "feature.flag".to_string(),
            depends_on_value: "enabled".to_string(),
            applies_to_pattern: "exact.key".to_string(),
        };

        // Should match exactly
        assert!(rule.matches_pattern("exact.key"));
    }

    /// Test that exact patterns don't match nested variations
    #[test]
    fn test_exact_pattern_rejects_nested() {
        let rule = ConditionalVisibility {
            base_setting: "feature.flag".to_string(),
            depends_on_value: "enabled".to_string(),
            applies_to_pattern: "exact.key".to_string(),
        };

        // Should NOT match nested variations
        assert!(!rule.matches_pattern("exact.key.nested"));
        assert!(!rule.matches_pattern("exact"));
        assert!(!rule.matches_pattern("exact.key.sub"));
    }

    // ============================================================================
    // MULTI-LEVEL NESTED PATTERN TESTS
    // ============================================================================

    /// Test wildcard patterns with deeper nesting
    #[test]
    fn test_deeply_nested_wildcard_pattern() {
        let rule = ConditionalVisibility {
            base_setting: "database.type".to_string(),
            depends_on_value: "postgres".to_string(),
            applies_to_pattern: "database.postgres.*".to_string(),
        };

        // Should match various depths of postgres settings
        assert!(rule.matches_pattern("database.postgres.host"));
        assert!(rule.matches_pattern("database.postgres.port"));
        assert!(rule.matches_pattern("database.postgres.connection.pool.size"));

        // Should NOT match different database types or prefix alone
        assert!(!rule.matches_pattern("database.mysql.host"));
        assert!(!rule.matches_pattern("database.postgres")); // Prefix alone doesn't match
    }

    /// Test MCP integration patterns
    #[test]
    fn test_spark_mcp_conditional_pattern() {
        let rule = ConditionalVisibility {
            base_setting: "spark.use_mcp".to_string(),
            depends_on_value: "true".to_string(),
            applies_to_pattern: "spark.mcp.*".to_string(),
        };

        // MCP-specific settings
        assert!(rule.matches_pattern("spark.mcp.enabled"));
        assert!(rule.matches_pattern("spark.mcp.port"));
        assert!(rule.matches_pattern("spark.mcp.host"));

        // Non-MCP settings
        assert!(!rule.matches_pattern("spark.history_server_url"));
        assert!(!rule.matches_pattern("spark.use_mcp"));
    }

    // ============================================================================
    // EDGE CASES
    // ============================================================================

    /// Test empty string handling
    #[test]
    fn test_empty_pattern_string() {
        let rule = ConditionalVisibility {
            base_setting: "key".to_string(),
            depends_on_value: "value".to_string(),
            applies_to_pattern: "".to_string(),
        };

        // Empty pattern should only match empty key
        assert!(rule.matches_pattern(""));
        assert!(!rule.matches_pattern("a"));
        assert!(!rule.matches_pattern("anything"));
    }

    /// Test single-level key patterns
    #[test]
    fn test_single_level_key_pattern() {
        let rule = ConditionalVisibility {
            base_setting: "mode".to_string(),
            depends_on_value: "advanced".to_string(),
            applies_to_pattern: "advanced.*".to_string(),
        };

        // Settings starting with "advanced."
        assert!(rule.matches_pattern("advanced.setting"));
        assert!(rule.matches_pattern("advanced.feature"));

        // Prefix alone or different prefix don't match
        assert!(!rule.matches_pattern("advanced")); // Prefix alone
        assert!(!rule.matches_pattern("basic.setting")); // Different prefix
    }

    /// Test pattern with multiple dots
    #[test]
    fn test_pattern_with_multiple_dots() {
        let rule = ConditionalVisibility {
            base_setting: "auth.type".to_string(),
            depends_on_value: "oauth2".to_string(),
            applies_to_pattern: "auth.oauth2.provider.*".to_string(),
        };

        // Matches settings under auth.oauth2.provider.
        assert!(rule.matches_pattern("auth.oauth2.provider.github"));
        assert!(rule.matches_pattern("auth.oauth2.provider.google"));

        // Prefix alone or wrong path don't match
        assert!(!rule.matches_pattern("auth.oauth2.provider")); // Prefix alone
        assert!(!rule.matches_pattern("auth.oauth.provider.github")); // Different path
    }

    // ============================================================================
    // INTEGRATION WITH SETTINGMETADATA
    // ============================================================================

    /// Test ConditionalVisibility as part of SettingMetadata
    #[test]
    fn test_conditional_setting_metadata_integration() {
        let metadata = SettingMetadata {
            key: "llm.ollama.base_url".to_string(),
            label: "Ollama Base URL".to_string(),
            description: "Base URL for Ollama service".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: Some(json!("http://localhost:11434")),
            constraints: vec![Constraint::Required],
            visibility: Visibility::Public,
            group: Some("llm".to_string()),
            conditional: Some(ConditionalVisibility {
                base_setting: "llm.provider".to_string(),
                depends_on_value: "ollama".to_string(),
                applies_to_pattern: "llm.ollama.*".to_string(),
            }),
        };

        // Verify metadata contains the conditional
        assert!(metadata.conditional.is_some());

        let cond = metadata.conditional.unwrap();
        assert_eq!(cond.base_setting, "llm.provider");
        assert_eq!(cond.depends_on_value, "ollama");
        assert!(cond.matches_pattern(&metadata.key));
    }

    /// Test base setting without conditional
    #[test]
    fn test_base_setting_without_conditional() {
        let metadata = SettingMetadata {
            key: "llm.provider".to_string(),
            label: "LLM Provider".to_string(),
            description: "Which LLM provider to use".to_string(),
            setting_type: SettingType::Enum {
                variants: vec!["ollama".to_string(), "openai".to_string(), "anthropic".to_string()],
            },
            default: Some(json!("ollama")),
            constraints: vec![Constraint::Required],
            visibility: Visibility::Public,
            group: Some("llm".to_string()),
            conditional: None,
        };

        // Base settings should not have conditional
        assert!(metadata.conditional.is_none());
    }

    /// Test multiple conditional settings for same base
    #[test]
    fn test_multiple_conditionals_for_same_base_setting() {
        let ollama_url = SettingMetadata {
            key: "llm.ollama.base_url".to_string(),
            label: "Ollama Base URL".to_string(),
            description: "".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: Some(json!("http://localhost:11434")),
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
            conditional: Some(ConditionalVisibility {
                base_setting: "llm.provider".to_string(),
                depends_on_value: "ollama".to_string(),
                applies_to_pattern: "llm.ollama.*".to_string(),
            }),
        };

        let openai_key = SettingMetadata {
            key: "llm.openai.api_key".to_string(),
            label: "OpenAI API Key".to_string(),
            description: "".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: Some(json!("")),
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
            conditional: Some(ConditionalVisibility {
                base_setting: "llm.provider".to_string(),
                depends_on_value: "openai".to_string(),
                applies_to_pattern: "llm.openai.*".to_string(),
            }),
        };

        // Both reference same base setting but different values
        assert_eq!(
            ollama_url.conditional.as_ref().unwrap().base_setting,
            openai_key.conditional.as_ref().unwrap().base_setting
        );

        // But have different depends_on_value
        assert_ne!(
            ollama_url.conditional.as_ref().unwrap().depends_on_value,
            openai_key.conditional.as_ref().unwrap().depends_on_value
        );
    }

    // ============================================================================
    // CLONING AND EQUALITY TESTS
    // ============================================================================

    /// Test that ConditionalVisibility can be cloned
    #[test]
    fn test_conditional_visibility_clone() {
        let rule1 = ConditionalVisibility {
            base_setting: "llm.provider".to_string(),
            depends_on_value: "ollama".to_string(),
            applies_to_pattern: "llm.ollama.*".to_string(),
        };

        let rule2 = rule1.clone();

        assert_eq!(rule1, rule2);
        assert_eq!(rule1.base_setting, rule2.base_setting);
        assert_eq!(rule1.depends_on_value, rule2.depends_on_value);
        assert_eq!(rule1.applies_to_pattern, rule2.applies_to_pattern);
    }

    /// Test that equal ConditionalVisibility rules are equal
    #[test]
    fn test_conditional_visibility_equality() {
        let rule1 = ConditionalVisibility {
            base_setting: "llm.provider".to_string(),
            depends_on_value: "ollama".to_string(),
            applies_to_pattern: "llm.ollama.*".to_string(),
        };

        let rule2 = ConditionalVisibility {
            base_setting: "llm.provider".to_string(),
            depends_on_value: "ollama".to_string(),
            applies_to_pattern: "llm.ollama.*".to_string(),
        };

        assert_eq!(rule1, rule2);
    }

    /// Test that different ConditionalVisibility rules are not equal
    #[test]
    fn test_conditional_visibility_inequality() {
        let rule1 = ConditionalVisibility {
            base_setting: "llm.provider".to_string(),
            depends_on_value: "ollama".to_string(),
            applies_to_pattern: "llm.ollama.*".to_string(),
        };

        let rule2 = ConditionalVisibility {
            base_setting: "llm.provider".to_string(),
            depends_on_value: "openai".to_string(),
            applies_to_pattern: "llm.openai.*".to_string(),
        };

        assert_ne!(rule1, rule2);
    }

    // ============================================================================
    // REAL-WORLD SCENARIO TESTS
    // ============================================================================

    /// Test the exact scenario: llm.ollama.* conditional on llm.provider = ollama
    #[test]
    fn test_scenario_ollama_provider_visibility() {
        // Scenario: Show ollama settings when provider is set to ollama

        let provider_setting = ConditionalVisibility {
            base_setting: "llm.provider".to_string(),
            depends_on_value: "ollama".to_string(),
            applies_to_pattern: "llm.ollama.*".to_string(),
        };

        // All ollama settings should match
        assert!(provider_setting.matches_pattern("llm.ollama.base_url"));
        assert!(provider_setting.matches_pattern("llm.ollama.model"));
        assert!(provider_setting.matches_pattern("llm.ollama.temperature"));
        assert!(provider_setting.matches_pattern("llm.ollama.max_tokens"));
        assert!(provider_setting.matches_pattern("llm.ollama.request_timeout"));

        // Other providers should not match
        assert!(!provider_setting.matches_pattern("llm.openai.api_key"));
        assert!(!provider_setting.matches_pattern("llm.anthropic.api_key"));

        // Base provider setting should not match
        assert!(!provider_setting.matches_pattern("llm.provider"));
    }

    /// Test database conditional visibility pattern
    #[test]
    fn test_scenario_database_postgres_visibility() {
        let postgres_rule = ConditionalVisibility {
            base_setting: "database.type".to_string(),
            depends_on_value: "postgres".to_string(),
            applies_to_pattern: "database.postgres.*".to_string(),
        };

        // Postgres-specific settings
        assert!(postgres_rule.matches_pattern("database.postgres.host"));
        assert!(postgres_rule.matches_pattern("database.postgres.port"));
        assert!(postgres_rule.matches_pattern("database.postgres.user"));
        assert!(postgres_rule.matches_pattern("database.postgres.password"));

        // MySQL settings should not match
        assert!(!postgres_rule.matches_pattern("database.mysql.host"));
        assert!(!postgres_rule.matches_pattern("database.mysql.port"));
    }

    /// Test future expansion with spark.use_mcp
    #[test]
    fn test_scenario_future_spark_mcp_expansion() {
        // Future scenario: Conditionally show MCP settings
        let mcp_rule = ConditionalVisibility {
            base_setting: "spark.use_mcp".to_string(),
            depends_on_value: "true".to_string(),
            applies_to_pattern: "spark.mcp.*".to_string(),
        };

        // MCP settings should match when enabled
        assert!(mcp_rule.matches_pattern("spark.mcp.enabled"));
        assert!(mcp_rule.matches_pattern("spark.mcp.port"));
        assert!(mcp_rule.matches_pattern("spark.mcp.host"));
        assert!(mcp_rule.matches_pattern("spark.mcp.connection.timeout"));

        // Non-MCP settings should not match
        assert!(!mcp_rule.matches_pattern("spark.history_server_url"));
        assert!(!mcp_rule.matches_pattern("spark.use_mcp"));
    }

    // ============================================================================
    // DEBUG AND DISPLAY TESTS
    // ============================================================================

    /// Test Debug implementation for ConditionalVisibility
    #[test]
    fn test_conditional_visibility_debug() {
        let rule = ConditionalVisibility {
            base_setting: "llm.provider".to_string(),
            depends_on_value: "ollama".to_string(),
            applies_to_pattern: "llm.ollama.*".to_string(),
        };

        let debug_str = format!("{:?}", rule);
        assert!(debug_str.contains("base_setting"));
        assert!(debug_str.contains("depends_on_value"));
        assert!(debug_str.contains("applies_to_pattern"));
    }

    /// Test string representation includes key information
    #[test]
    fn test_conditional_visibility_string_representation() {
        let rule = ConditionalVisibility {
            base_setting: "llm.provider".to_string(),
            depends_on_value: "ollama".to_string(),
            applies_to_pattern: "llm.ollama.*".to_string(),
        };

        // Verify all components are accessible
        assert_eq!(rule.base_setting, "llm.provider");
        assert_eq!(rule.depends_on_value, "ollama");
        assert_eq!(rule.applies_to_pattern, "llm.ollama.*");
    }
}
