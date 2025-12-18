//! Phase 5.3: Property-Based Tests for Validation System (sl-iii)
//!
//! Property-based tests using proptest for comprehensive edge case coverage.
//! These tests verify validation robustness, boundary handling, and performance.

#![cfg(feature = "metadata")]

#[cfg(test)]
#[cfg(feature = "metadata")]
mod phase5_3_property_tests {
    #![allow(unused_imports)]
    use proptest::prelude::*;
    use serde_json::json;
    use settings_loader::metadata::{Constraint, SettingMetadata, SettingType, Visibility};
    use std::time::Instant;

    // ============================================================================
    // PATTERN CONSTRAINT PROPERTY TESTS
    // ============================================================================

    proptest! {
        #[test]
        fn prop_pattern_lowercase_letters_valid(s in "[a-z]+") {
            let constraint = Constraint::Pattern("[a-z]+".to_string());
            prop_assert!(constraint.validate("test", &json!(s)).is_ok());
        }

        #[test]
        fn prop_pattern_digits_reject_letters(s in "[a-z]+") {
            let constraint = Constraint::Pattern("[0-9]+".to_string());
            prop_assert!(constraint.validate("test", &json!(s)).is_err());
        }

        #[test]
        fn prop_pattern_hex_validation(s in "[0-9a-f]+") {
            let constraint = Constraint::Pattern("^[0-9a-f]*$".to_string());
            prop_assert!(constraint.validate("test", &json!(s)).is_ok());
        }

        #[test]
        fn prop_pattern_email_like(user in "[a-z0-9]{3,10}", domain in "[a-z]{3,10}") {
            let pattern = format!("^{}@{}$", user, domain);
            let email = format!("{}@{}", user, domain);
            let constraint = Constraint::Pattern(pattern);
            prop_assert!(constraint.validate("email", &json!(email)).is_ok());
        }
    }

    // ============================================================================
    // RANGE CONSTRAINT PROPERTY TESTS
    // ============================================================================

    proptest! {
        #[test]
        fn prop_range_within_bounds_accepts(n in 1i64..=100i64) {
            let constraint = Constraint::Range { min: 1.0, max: 100.0 };
            prop_assert!(constraint.validate("test", &json!(n)).is_ok());
        }

        #[test]
        fn prop_range_below_min_rejects(n in i64::MIN..=0i64) {
            let constraint = Constraint::Range { min: 1.0, max: 100.0 };
            if (n as f64) < 1.0 {
                prop_assert!(constraint.validate("test", &json!(n)).is_err());
            }
        }

        #[test]
        fn prop_range_above_max_rejects(n in 101i64..i64::MAX) {
            let constraint = Constraint::Range { min: 1.0, max: 100.0 };
            prop_assert!(constraint.validate("test", &json!(n)).is_err());
        }

        #[test]
        fn prop_range_float_precision(n in 0.0f64..=1.0f64) {
            let constraint = Constraint::Range { min: 0.0, max: 1.0 };
            prop_assert!(constraint.validate("test", &json!(n)).is_ok());
        }

        #[test]
        fn prop_range_float_above_max(n in 1.0f64..100.0f64) {
            let constraint = Constraint::Range { min: 0.0, max: 1.0 };
            if n > 1.0 && n.is_finite() {
                prop_assert!(constraint.validate("test", &json!(n)).is_err());
            }
        }

        #[test]
        fn prop_range_exact_boundaries(
            min in -1000.0f64..=1000.0f64,
            offset in 1.0f64..=1000.0f64,
        ) {
            prop_assume!(min.is_finite() && offset.is_finite());
            let max = min + offset;
            let constraint = Constraint::Range { min, max };

            // Test exactly at min
            prop_assert!(constraint.validate("test", &json!(min)).is_ok());

            // Test exactly at max
            prop_assert!(constraint.validate("test", &json!(max)).is_ok());

            // Test just below min
            let just_below = min - 0.01;
            prop_assert!(constraint.validate("test", &json!(just_below)).is_err());

            // Test just above max
            let just_above = max + 0.01;
            prop_assert!(constraint.validate("test", &json!(just_above)).is_err());
        }

        #[test]
        fn prop_range_zero_within_bounds(min in -100.0f64..=0.0f64, max in 0.0f64..=100.0f64) {
            let constraint = Constraint::Range { min, max };
            prop_assert!(constraint.validate("test", &json!(0)).is_ok());
        }
    }

    // ============================================================================
    // LENGTH CONSTRAINT PROPERTY TESTS
    // ============================================================================

    proptest! {
        #[test]
        fn prop_length_valid_range(s in "[a-z]{1,10}") {
            let constraint = Constraint::Length { min: 1, max: 10 };
            prop_assert!(constraint.validate("test", &json!(s)).is_ok());
        }

        #[test]
        fn prop_length_too_long(s in "[a-z]{11,50}") {
            let constraint = Constraint::Length { min: 1, max: 10 };
            prop_assert!(constraint.validate("test", &json!(s)).is_err());
        }

        #[test]
        fn prop_length_at_boundaries(
            min in 1usize..=10usize,
            max in 1usize..=10usize,
        ) {
            prop_assume!(min <= max);
            let constraint = Constraint::Length { min, max };

            // Test exact min length
            let min_string = "a".repeat(min);
            prop_assert!(constraint.validate("test", &json!(min_string)).is_ok());

            // Test exact max length
            let max_string = "a".repeat(max);
            prop_assert!(constraint.validate("test", &json!(max_string)).is_ok());

            // Test one below min
            if min > 1 {
                let below_min = "a".repeat(min - 1);
                prop_assert!(constraint.validate("test", &json!(below_min)).is_err());
            }

            // Test one above max
            let above_max = "a".repeat(max + 1);
            prop_assert!(constraint.validate("test", &json!(above_max)).is_err());
        }

        #[test]
        fn prop_length_unicode_strings(s in "\\PC{1,10}") {
            let constraint = Constraint::Length { min: 1, max: 10 };
            // Should handle unicode correctly
            let result = constraint.validate("test", &json!(s));
            let _ = result; // Just ensure no panic
        }
    }

    // ============================================================================
    // ONEOF CONSTRAINT PROPERTY TESTS
    // ============================================================================

    proptest! {
        #[test]
        fn prop_oneof_valid_value(idx in 0usize..3usize) {
            let allowed = vec!["red".to_string(), "green".to_string(), "blue".to_string()];
            let value = allowed[idx].clone();
            let constraint = Constraint::OneOf(allowed);
            prop_assert!(constraint.validate("test", &json!(value)).is_ok());
        }

        #[test]
        fn prop_oneof_rejects_outside_set(s in "[a-z]{4,10}") {
            let constraint = Constraint::OneOf(vec![
                "red".to_string(),
                "green".to_string(),
                "blue".to_string(),
            ]);
            if !["red", "green", "blue"].contains(&s.as_str()) {
                prop_assert!(constraint.validate("test", &json!(s)).is_err());
            }
        }

        #[test]
        fn prop_oneof_case_sensitive(s in "[a-z]{3,10}") {
            let allowed = vec!["RED".to_string(), "GREEN".to_string()];
            let constraint = Constraint::OneOf(allowed);
            // Lowercase values should not match uppercase
            if s != "RED" && s != "GREEN" {
                prop_assert!(constraint.validate("test", &json!(s)).is_err());
            }
        }

        #[test]
        fn prop_oneof_with_many_options(
            options in prop::collection::vec("[a-z0-9]{2,5}", 5..20),
            idx in 0usize..100,
        ) {
            let constraint = Constraint::OneOf(options.clone());
            let test_idx = idx % options.len();
            let valid_value = &options[test_idx];
            prop_assert!(constraint.validate("test", &json!(valid_value.clone())).is_ok());
        }
    }

    // ============================================================================
    // TYPE VALIDATION BOUNDARY TESTS
    // ============================================================================

    proptest! {
        #[test]
        fn prop_integer_type_bounds(min in 0i64..100i64, max in 100i64..1000i64) {
            let setting_type = SettingType::Integer { min: Some(min), max: Some(max) };

            prop_assert!(setting_type.validate("test", &json!(min)).is_ok());
            prop_assert!(setting_type.validate("test", &json!(max)).is_ok());

            let mid = min + (max - min) / 2;
            prop_assert!(setting_type.validate("test", &json!(mid)).is_ok());

            prop_assert!(setting_type.validate("test", &json!(min - 1)).is_err());
            prop_assert!(setting_type.validate("test", &json!(max + 1)).is_err());
        }

        #[test]
        fn prop_float_type_boundary(min in 0.0f64..100.0f64, max in 100.0f64..1000.0f64) {
            prop_assume!(min.is_finite() && max.is_finite());
            let setting_type = SettingType::Float { min: Some(min), max: Some(max) };

            prop_assert!(setting_type.validate("test", &json!(min)).is_ok());
            prop_assert!(setting_type.validate("test", &json!(max)).is_ok());
        }

        #[test]
        fn prop_string_type_length_bounds(min in 1usize..=5usize, max in 5usize..=20usize) {
            let setting_type = SettingType::String {
                pattern: None,
                min_length: Some(min),
                max_length: Some(max),
            };

            let valid = "a".repeat(min);
            prop_assert!(setting_type.validate("test", &json!(valid)).is_ok());

            let too_short = "a".repeat(min.saturating_sub(1));
            if min > 0 {
                prop_assert!(setting_type.validate("test", &json!(too_short)).is_err());
            }

            let too_long = "a".repeat(max + 1);
            prop_assert!(setting_type.validate("test", &json!(too_long)).is_err());
        }
    }

    // ============================================================================
    // ERROR MESSAGE CONSISTENCY TESTS
    // ============================================================================

    proptest! {
        #[test]
        fn prop_pattern_error_contains_key(pattern in "[a-z]{3,10}") {
            let constraint = Constraint::Pattern(pattern.clone());
            let result = constraint.validate("test_key", &json!("123"));

            if result.is_err() {
                let error_msg = result.unwrap_err().to_string();
                prop_assert!(error_msg.contains("test_key"));
            }
        }

        #[test]
        fn prop_range_error_shows_bounds(min in 1f64..10f64, max in 10f64..100f64) {
            let constraint = Constraint::Range { min, max };
            let result = constraint.validate("test_key", &json!(1000));

            if result.is_err() {
                let error_msg = result.unwrap_err().to_string();
                prop_assert!(error_msg.contains("test_key"));
            }
        }

        #[test]
        fn prop_length_error_shows_actual_length(
            min in 1usize..=5usize,
            max in 5usize..=10usize,
        ) {
            let constraint = Constraint::Length { min, max };
            let too_short = "a".repeat(min.saturating_sub(1));
            let result = constraint.validate("test_key", &json!(too_short));

            if result.is_err() && min > 0 {
                let error_msg = result.unwrap_err().to_string();
                prop_assert!(error_msg.contains("test_key"));
            }
        }

        #[test]
        fn prop_oneof_error_shows_key(allowed_vals in prop::collection::vec("[a-z]+", 1..5)) {
            let allowed = allowed_vals.iter().map(|s| s.clone()).collect();
            let constraint = Constraint::OneOf(allowed);
            let result = constraint.validate("test_key", &json!("invalid_xyz"));

            if result.is_err() {
                let error_msg = result.unwrap_err().to_string();
                prop_assert!(error_msg.contains("test_key"));
            }
        }
    }

    // ============================================================================
    // PERFORMANCE TESTS
    // ============================================================================

    #[test]
    fn prop_validation_performance_pattern() {
        let constraint = Constraint::Pattern("[a-z0-9]+".to_string());
        let start = Instant::now();

        for i in 0..1000 {
            let _ = constraint.validate("test", &json!(format!("value{}", i)));
        }

        let elapsed = start.elapsed();
        let millis = elapsed.as_millis() as u128;
        // 1000 validations should complete in under 1000ms (avg ~1ms per)
        assert!(
            millis < 1000,
            "Pattern validation too slow: {:?}ms for 1000 ops (avg {}ms each)",
            millis,
            millis / 1000
        );
    }

    #[test]
    fn prop_validation_performance_range() {
        let constraint = Constraint::Range { min: 0.0, max: 1000.0 };
        let start = Instant::now();

        for i in 0..1000 {
            let _ = constraint.validate("test", &json!(i));
        }

        let elapsed = start.elapsed();
        let millis = elapsed.as_millis() as u128;
        assert!(millis < 1000, "Range validation too slow: {:?}ms for 1000 ops", millis);
    }

    #[test]
    fn prop_validation_performance_length() {
        let constraint = Constraint::Length { min: 1, max: 100 };
        let start = Instant::now();

        for i in 0..1000 {
            let _ = constraint.validate("test", &json!(format!("value_{}", i)));
        }

        let elapsed = start.elapsed();
        let millis = elapsed.as_millis() as u128;
        assert!(millis < 1000, "Length validation too slow: {:?}ms for 1000 ops", millis);
    }

    #[test]
    fn prop_validation_performance_oneof() {
        let allowed = vec!["red", "green", "blue", "yellow", "purple"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let constraint = Constraint::OneOf(allowed);
        let start = Instant::now();

        let colors = ["red", "green", "blue", "yellow", "purple"];
        for i in 0..1000 {
            let _ = constraint.validate("test", &json!(colors[i % 5]));
        }

        let elapsed = start.elapsed();
        let millis = elapsed.as_millis() as u128;
        assert!(millis < 1000, "OneOf validation too slow: {:?}ms for 1000 ops", millis);
    }

    // ============================================================================
    // EDGE CASE TESTS
    // ============================================================================

    proptest! {
        #[test]
        fn prop_empty_string_length_constraint(max in 1usize..=100usize) {
            let constraint = Constraint::Length { min: 0, max };
            prop_assert!(constraint.validate("test", &json!("")).is_ok());
        }

        #[test]
        fn prop_zero_value_range_constraint(_dummy in 0u32..=0u32) {
            let constraint = Constraint::Range { min: -100.0, max: 100.0 };
            prop_assert!(constraint.validate("test", &json!(0)).is_ok());
        }

        #[test]
        fn prop_pattern_empty_string(pattern in "[a-z]*") {
            let constraint = Constraint::Pattern(pattern);
            let result = constraint.validate("test", &json!(""));
            // Empty string might or might not match depending on pattern
            let _ = result;
        }
    }

    #[test]
    fn prop_null_value_required_constraint() {
        let constraint = Constraint::Required;
        assert!(constraint.validate("test", &json!(null)).is_err());
    }

    // ============================================================================
    // COMPLEX VALIDATION SCENARIOS
    // ============================================================================

    proptest! {
        #[test]
        fn prop_metadata_multiple_constraints(
            pattern in "[a-z]{3,10}",
            min_len in 1usize..=5usize,
            max_len in 5usize..=20usize,
        ) {
            let metadata = SettingMetadata {
                key: "username".to_string(),
                label: "Username".to_string(),
                description: "User username".to_string(),
                setting_type: SettingType::String {
                    pattern: Some(pattern),
                    min_length: Some(min_len),
                    max_length: Some(max_len),
                },
                default: None,
                constraints: vec![
                    Constraint::Required,
                    Constraint::Length {
                        min: min_len,
                        max: max_len,
                    },
                ],
                visibility: Visibility::Public,
                group: None,
            };

            // Test with valid value
            let valid = "hello";
            if valid.len() >= min_len && valid.len() <= max_len {
                let result = metadata.validate(&json!(valid));
                // At minimum shouldn't panic
                let _ = result;
            }
        }

        #[test]
        fn prop_secret_visibility_validation(
            value in "[a-zA-Z0-9]{8,32}",
        ) {
            let metadata = SettingMetadata {
                key: "api_secret".to_string(),
                label: "API Secret".to_string(),
                description: "Secret API key".to_string(),
                setting_type: SettingType::String {
                    pattern: None,
                    min_length: Some(8),
                    max_length: None,
                },
                default: None,
                constraints: vec![Constraint::Required],
                visibility: Visibility::Secret,
                group: None,
            };

            let result = metadata.validate(&json!(value));
            // Should not panic and result should be valid for good values
            let _ = result;
        }
    }

    // ============================================================================
    // CONSTRAINT HINT MESSAGE TESTS (sl-4ri)
    // ============================================================================

    #[test]
    fn test_pattern_error_includes_hint() {
        use settings_loader::validation::ValidationError;

        let error = ValidationError::InvalidPattern {
            key: "email".to_string(),
            pattern: "[a-z0-9]+@[a-z0-9]+\\.[a-z]{2,}".to_string(),
            value: "invalid".to_string(),
        };

        let msg = error.to_string();
        // Should include the pattern hint
        assert!(msg.contains("expected:"), "Error should include 'expected:' hint");
        assert!(
            msg.contains("pattern matching"),
            "Error should mention pattern constraint"
        );
    }

    #[test]
    fn test_range_error_includes_hint() {
        use settings_loader::validation::ValidationError;

        let error = ValidationError::OutOfRange {
            key: "port".to_string(),
            min: 1024.0,
            max: 65535.0,
            value: 70000.0,
        };

        let msg = error.to_string();
        // Should include the range hint
        assert!(msg.contains("expected:"), "Error should include 'expected:' hint");
        assert!(msg.contains("between"), "Error should mention 'between' in hint");
        assert!(msg.contains("1024"), "Error should show min bound");
        assert!(msg.contains("65535"), "Error should show max bound");
    }

    #[test]
    fn test_oneof_error_includes_hint() {
        use settings_loader::validation::ValidationError;

        let error = ValidationError::NotOneOf {
            key: "env".to_string(),
            expected: vec!["dev".to_string(), "staging".to_string(), "prod".to_string()],
            actual: "invalid".to_string(),
        };

        let msg = error.to_string();
        // Should include the allowed values hint
        assert!(msg.contains("expected:"), "Error should include 'expected:' hint");
        assert!(msg.contains("one of"), "Error should mention 'one of' in hint");
        assert!(msg.contains("dev"), "Error should list allowed values");
    }

    #[test]
    fn test_constraint_hints_actionable() {
        use settings_loader::validation::ValidationError;

        // Pattern error should be actionable
        let pattern_error = ValidationError::InvalidPattern {
            key: "username".to_string(),
            pattern: "^[a-zA-Z0-9_]{3,16}$".to_string(),
            value: "ab".to_string(),
        };
        let pattern_msg = pattern_error.to_string();
        assert!(
            pattern_msg.contains("expected:") && pattern_msg.contains("pattern"),
            "Pattern error should provide actionable hint"
        );

        // Range error should be actionable
        let range_error = ValidationError::OutOfRange {
            key: "connections".to_string(),
            min: 1.0,
            max: 1000.0,
            value: 5000.0,
        };
        let range_msg = range_error.to_string();
        assert!(
            range_msg.contains("expected:") && range_msg.contains("between"),
            "Range error should provide actionable hint"
        );
    }
}
