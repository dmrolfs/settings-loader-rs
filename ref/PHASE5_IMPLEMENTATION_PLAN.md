# Phase 5: Settings Metadata & Introspection - Implementation Plan

**Epic**: sl-wnc  
**Phases**: 6 implementation phases (5.1 - 5.6)  
**Approach**: Test-Driven Development (TDD)  
**Created**: December 18, 2025

---

## Overview

Phase 5 adds runtime introspection capabilities through a metadata system. Implementation is broken into 6 sub-phases, each with its own test suite, implementation, and review tasks.

**Total Estimated Timeline**: 7-10 days
- Phases 5.1-5.4: 5-7 days (core + validation)
- Phase 5.5: 2-3 days (optional proc-macro)
- Phase 5.6: 1 day (final review)

---

## Phase Structure

Each phase follows this pattern:

1. **TDD Test Suite Task**: Write comprehensive tests (RED phase)
2. **Implementation Task(s)**: Implement features to make tests pass (GREEN phase)
3. **Architecture Review Task**: Review for maintainability, modularity, best practices
4. **Test Enhancement Task**: Improve test coverage and quality

---

## Phase 5.1: Core Metadata Types

**Duration**: 1-2 days  
**Focus**: Foundation types without validation logic

### Deliverables

- `src/metadata.rs` module
- Core types: `SettingMetadata`, `SettingType`, `Constraint`, `Visibility`
- `ConfigSchema` and `SettingGroup`

### Tasks

#### Task 5.1.1: TDD Test Suite (RED)
**Beads**: sl-426 ✅  
**Type**: task  
**Priority**: 2

**Description**:
Write comprehensive test suite for Phase 5.1 core types before implementation.

**Acceptance Criteria**:
- [ ] Test file `tests/phase5_1_core_metadata_tests.rs` created
- [ ] 15+ tests written covering:
  - SettingMetadata construction
  - SettingType enum variants
  - Constraint enum variants
  - Visibility enum usage
  - ConfigSchema construction
  - SettingGroup organization
- [ ] Tests compile but fail (RED phase)
- [ ] Tests use stub implementations

**Test Coverage**:
```rust
// tests/phase5_1_core_metadata_tests.rs

#[test]
fn test_setting_metadata_construction()
#[test]
fn test_setting_type_string_with_constraints()
#[test]
fn test_setting_type_integer_with_range()
#[test]
fn test_setting_type_float_with_range()
#[test]
fn test_setting_type_boolean()
#[test]
fn test_setting_type_duration()
#[test]
fn test_setting_type_path()
#[test]
fn test_setting_type_url_with_schemes()
#[test]
fn test_setting_type_enum_with_variants()
#[test]
fn test_setting_type_secret()
#[test]
fn test_setting_type_array()
#[test]
fn test_setting_type_object()
#[test]
fn test_constraint_pattern()
#[test]
fn test_constraint_range()
#[test]
fn test_constraint_length()
#[test]
fn test_constraint_required()
#[test]
fn test_constraint_one_of()
#[test]
fn test_visibility_variants()
#[test]
fn test_config_schema_construction()
#[test]
fn test_setting_group_organization()
```

---

#### Task 5.1.2: Implement Core Types (GREEN)
**Beads**: sl-eh6 ✅  
**Type**: task  
**Priority**: 2  
**Depends**: sl-426

**Description**:
Implement core metadata types in `src/metadata.rs`.

**Acceptance Criteria**:
- [ ] `src/metadata.rs` created with all types
- [ ] SettingMetadata struct fully implemented
- [ ] SettingType enum with all variants
- [ ] Constraint enum with all variants
- [ ] Visibility enum implemented
- [ ] ConfigSchema struct implemented
- [ ] SettingGroup struct implemented
- [ ] All Phase 5.1 tests passing (GREEN)
- [ ] Module exported in `src/lib.rs`
- [ ] Documentation comments on all public items
- [ ] 0 clippy warnings

**Implementation Steps**:
1. Create `src/metadata.rs`
2. Implement Visibility enum (simplest)
3. Implement Constraint enum
4. Implement SettingType enum (complex)
5. Implement SettingMetadata struct
6. Implement SettingGroup struct
7. Implement ConfigSchema struct
8. Add Clone, Debug derives where appropriate
9. Export module in lib.rs

---

#### Task 5.1.3: Architecture Review (REFACTOR)
**Beads**: sl-8wc ✅  
**Type**: task  
**Priority**: 2  
**Depends**: sl-eh6

**Description**:
Review Phase 5.1 implementation for improvements in maintainability, modularity, and idiomatic Rust practices.

**Acceptance Criteria**:
- [ ] Code reviewed for idiomatic Rust patterns
- [ ] Type definitions follow Rust API guidelines
- [ ] Clone/Copy traits appropriately applied
- [ ] serde Serialize/Deserialize added where needed
- [ ] Module organization reviewed
- [ ] Documentation reviewed for clarity
- [ ] Examples added to documentation
- [ ] Edge cases identified and documented
- [ ] All tests still passing after refactor

**Review Checklist**:
- [ ] Are enums marked `#[non_exhaustive]` where appropriate?
- [ ] Are all types properly documented?
- [ ] Should any types be `Copy` instead of just `Clone`?
- [ ] Are field names consistent with Rust conventions?
- [ ] Should any fields be `pub(crate)` instead of `pub`?
- [ ] Are there builder patterns that would improve ergonomics?

---

#### Task 5.1.4: Test Enhancement (TEST QUALITY)
**Beads**: sl-vnd ✅  
**Type**: task  
**Priority**: 2  
**Depends**: sl-8wc

**Description**:
Enhance Phase 5.1 test suite to improve coverage and quality.

**Acceptance Criteria**:
- [ ] Additional edge case tests added
- [ ] Property-based tests considered (if applicable)
- [ ] Test helper functions extracted for reuse
- [ ] Test documentation improved
- [ ] Coverage gaps identified and filled
- [ ] 5+ additional tests added
- [ ] All tests passing

**Enhancement Areas**:
- [ ] Test SettingType with extreme values
- [ ] Test SettingMetadata with None/Optional fields
- [ ] Test ConfigSchema with empty settings list
- [ ] Test SettingGroup with duplicate keys
- [ ] Test Clone/Debug implementations
- [ ] Test serde serialization (if added)

---

## Phase 5.2: SettingsIntrospection Trait

**Duration**: 1 day  
**Focus**: Trait definition and default implementations

### Deliverables

- `src/introspection.rs` module (or in `src/metadata.rs`)
- `SettingsIntrospection` trait
- Default trait method implementations

### Tasks

#### Task 5.2.1: TDD Test Suite (RED)
**Beads**: sl-45l ✅  
**Type**: task  
**Priority**: 2  
**Depends**: Phase 5.1 complete (sl-vnd)

**Description**:
Write comprehensive test suite for SettingsIntrospection trait.

**Acceptance Criteria**:
- [ ] Test file `tests/phase5_2_introspection_tests.rs` created
- [ ] 10+ tests written covering:
  - Manual trait implementation
  - schema() method
  - metadata() method
  - metadata_for() lookup
  - validate_value() method
  - keys() list generation
  - settings_in_group() filtering
- [ ] Tests compile but fail (RED phase)
- [ ] Mock settings struct for testing

**Test Coverage**:
```rust
#[test]
fn test_manual_introspection_impl()
#[test]
fn test_schema_method_returns_config_schema()
#[test]
fn test_metadata_returns_static_array()
#[test]
fn test_metadata_for_finds_setting()
#[test]
fn test_metadata_for_returns_none_for_unknown()
#[test]
fn test_validate_value_accepts_valid()
#[test]
fn test_validate_value_rejects_invalid()
#[test]
fn test_keys_returns_all_setting_keys()
#[test]
fn test_settings_in_group_filters_correctly()
#[test]
fn test_settings_in_group_returns_empty_for_unknown()
```

---

#### Task 5.2.2: Implement Trait (GREEN)
**Beads**: sl-0d7 ✅  
**Type**: task  
**Priority**: 2  
**Depends**: sl-45l

**Description**:
Implement SettingsIntrospection trait with default methods.

**Acceptance Criteria**:
- [ ] SettingsIntrospection trait defined
- [ ] Required methods: schema(), metadata()
- [ ] Default methods: metadata_for(), keys(), settings_in_group()
- [ ] validate_value() with basic implementation
- [ ] Trait extends SettingsLoader
- [ ] All Phase 5.2 tests passing
- [ ] Documentation complete
- [ ] Example implementation shown in docs
- [ ] 0 clippy warnings

---

#### Task 5.2.3: Architecture Review (REFACTOR)
**Beads**: sl-3qk ✅  
**Type**: task  
**Priority**: 2  
**Depends**: sl-0d7

**Acceptance Criteria**:
- [ ] Trait design reviewed against Rust API guidelines
- [ ] Default method implementations optimized
- [ ] Should validate_value() have different signature?
- [ ] Are lifetimes optimal?
- [ ] Documentation clarity reviewed
- [ ] All tests still passing

---

#### Task 5.2.4: Test Enhancement (TEST QUALITY)
**Beads**: sl-cjw ✅  
**Type**: task  
**Priority**: 2  
**Depends**: sl-3qk

**Acceptance Criteria**:
- [ ] Additional edge case tests
- [ ] Test trait with complex nested settings
- [ ] Test trait with empty metadata
- [ ] Test performance with large schema
- [ ] 5+ additional tests added

---

## Phase 5.3: Validation Framework

**Duration**: 2 days  
**Focus**: Constraint validation and error handling

### Deliverables

- `src/validation.rs` module
- `ValidationError` type
- Validation logic for each SettingType
- Constraint checking implementations

### Tasks

#### Task 5.3.1: TDD Test Suite (RED)
**Beads**: sl-fff (to be created)  
**Type**: task  
**Priority**: 2  
**Depends**: Phase 5.2 complete

**Description**:
Write comprehensive test suite for validation framework.

**Acceptance Criteria**:
- [ ] Test file `tests/phase5_3_validation_tests.rs` created
- [ ] 20+ tests written covering:
  - ValidationError construction
  - Type validation for each SettingType
  - Constraint validation for each Constraint
  - Edge cases (None values, type mismatches)
  - Multiple constraint validation
  - Error message quality
- [ ] Tests compile but fail (RED phase)

**Test Coverage**:
```rust
// Type validation
#[test]
fn test_validate_string_type()
#[test]
fn test_validate_integer_type()
#[test]
fn test_validate_float_type()
#[test]
fn test_validate_boolean_type()
#[test]
fn test_validate_duration_type()
#[test]
fn test_validate_path_type()
#[test]
fn test_validate_url_type()
#[test]
fn test_validate_enum_type()
#[test]
fn test_validate_array_type()
#[test]
fn test_validate_object_type()

// Constraint validation
#[test]
fn test_validate_pattern_constraint()
#[test]
fn test_validate_range_constraint()
#[test]
fn test_validate_length_constraint()
#[test]
fn test_validate_required_constraint()
#[test]
fn test_validate_one_of_constraint()

// Edge cases
#[test]
fn test_validate_type_mismatch_error()
#[test]
fn test_validate_multiple_constraints()
#[test]
fn test_validate_none_value_with_required()
#[test]
fn test_validate_error_messages_clear()
```

---

#### Task 5.3.2: Implement Validation (GREEN)
**Beads**: sl-ggg (to be created)  
**Type**: task  
**Priority**: 2  
**Depends**: sl-fff

**Description**:
Implement validation framework with error handling.

**Acceptance Criteria**:
- [ ] `src/validation.rs` created
- [ ] ValidationError struct implemented
- [ ] SettingType::validate_type() implemented for all variants
- [ ] Constraint::validate() implemented for all variants
- [ ] SettingMetadata::validate() orchestration method
- [ ] Clear error messages for all validation failures
- [ ] All Phase 5.3 tests passing
- [ ] Documentation complete
- [ ] 0 clippy warnings

**Implementation Steps**:
1. Create ValidationError struct with Display impl
2. Implement SettingType::validate_type() for primitives
3. Implement SettingType::validate_type() for complex types
4. Implement Constraint::validate() for each variant
5. Implement SettingMetadata::validate() to orchestrate
6. Add helper methods for common patterns
7. Write error message formatting

---

#### Task 5.3.3: Architecture Review (REFACTOR)
**Beads**: sl-hhh (to be created)  
**Type**: task  
**Priority**: 2  
**Depends**: sl-ggg

**Acceptance Criteria**:
- [ ] Error handling patterns reviewed
- [ ] Should use thiserror for ValidationError?
- [ ] Validation performance reviewed
- [ ] Regex compilation handled efficiently?
- [ ] Error messages user-friendly?
- [ ] All tests still passing

---

#### Task 5.3.4: Test Enhancement (TEST QUALITY)
**Beads**: sl-iii (to be created)  
**Type**: task  
**Priority**: 2  
**Depends**: sl-hhh

**Acceptance Criteria**:
- [ ] Complex nested validation scenarios tested
- [ ] Performance tests for large values
- [ ] Unicode and edge case strings tested
- [ ] Validation error formatting tested
- [ ] 5+ additional tests added

---

## Phase 5.4: Integration & Examples

**Duration**: 1 day  
**Focus**: Integration with existing phases and real-world examples

### Deliverables

- Integration with Phase 4 (editing)
- Example implementations
- Real-world use case tests

### Tasks

#### Task 5.4.1: TDD Test Suite (RED)
**Beads**: sl-jjj (to be created)  
**Type**: task  
**Priority**: 2  
**Depends**: Phase 5.3 complete

**Description**:
Write integration test suite showing Phase 5 with other phases.

**Acceptance Criteria**:
- [ ] Test file `tests/phase5_4_integration_tests.rs` created
- [ ] 10+ tests covering:
  - Manual SettingsIntrospection implementation
  - Metadata + LayerEditor integration
  - Validation before save
  - TUI use case simulation
  - CLI help generation
  - Schema export
- [ ] Tests compile but fail (RED phase)

**Test Coverage**:
```rust
#[test]
fn test_manual_introspection_implementation()
#[test]
fn test_validation_before_editor_save()
#[test]
fn test_generate_tui_form_from_metadata()
#[test]
fn test_generate_cli_help_from_schema()
#[test]
fn test_filter_secrets_from_ui()
#[test]
fn test_group_settings_for_display()
#[test]
fn test_validate_all_settings_batch()
#[test]
fn test_introspection_with_multi_scope()
#[test]
fn test_backward_compatibility_no_introspection()
```

---

#### Task 5.4.2: Integration Implementation (GREEN)
**Beads**: sl-kkk (to be created)  
**Type**: task  
**Priority**: 2  
**Depends**: sl-jjj

**Description**:
Implement integration examples and ensure Phase 5 works with existing phases.

**Acceptance Criteria**:
- [ ] Example implementation in tests/
- [ ] Integration with LayerEditor demonstrated
- [ ] TUI form generation helper implemented
- [ ] CLI help generation helper implemented
- [ ] All Phase 5.4 tests passing
- [ ] Documentation includes integration examples
- [ ] 0 clippy warnings

---

#### Task 5.4.3: Architecture Review (REFACTOR)
**Beads**: sl-lll (to be created)  
**Type**: task  
**Priority**: 2  
**Depends**: sl-kkk

**Acceptance Criteria**:
- [ ] Integration patterns reviewed
- [ ] Helper functions extracted to utilities?
- [ ] Should add convenience methods to traits?
- [ ] Documentation integration reviewed
- [ ] All tests still passing

---

#### Task 5.4.4: Test Enhancement (TEST QUALITY)
**Beads**: sl-mmm (to be created)  
**Type**: task  
**Priority**: 2  
**Depends**: sl-lll

**Acceptance Criteria**:
- [ ] Real-world Turtle use case tested
- [ ] Performance benchmarks added
- [ ] Error path testing enhanced
- [ ] Documentation examples tested as doc tests
- [ ] 5+ additional tests added

---

## Phase 5.5: Proc-Macro (OPTIONAL)

**Duration**: 2-3 days  
**Focus**: Automatic metadata generation via derive macro

**Note**: This phase is optional and can be deferred to post-v0.22.0 if time constrained.

### Deliverables

- `settings-loader-derive` crate
- `#[derive(SettingsSchema)]` macro
- `#[setting(...)]` attribute parsing

### Tasks

#### Task 5.5.1: TDD Test Suite (RED)
**Beads**: sl-nnn (to be created)  
**Type**: task  
**Priority**: 3 (lower priority - optional)  
**Depends**: Phase 5.4 complete

**Description**:
Write test suite for proc-macro code generation.

**Acceptance Criteria**:
- [ ] Test directory `settings-loader-derive/tests/` created
- [ ] 15+ tests covering:
  - Basic derive macro usage
  - All #[setting(...)] attributes
  - Nested structs
  - Generic types
  - Error cases (invalid attributes)
- [ ] Tests use trybuild for compile-time testing
- [ ] Tests compile but fail (RED phase)

---

#### Task 5.5.2: Implement Proc-Macro (GREEN)
**Beads**: sl-ooo (to be created)  
**Type**: task  
**Priority**: 3  
**Depends**: sl-nnn

**Description**:
Implement derive macro for automatic metadata generation.

**Acceptance Criteria**:
- [ ] `settings-loader-derive` crate created
- [ ] Cargo.toml with proc-macro = true
- [ ] Parse struct definition with syn
- [ ] Parse #[setting(...)] attributes
- [ ] Generate SettingsIntrospection impl
- [ ] Handle all SettingType variants
- [ ] All Phase 5.5 tests passing
- [ ] Documentation complete
- [ ] 0 clippy warnings

---

#### Task 5.5.3: Architecture Review (REFACTOR)
**Beads**: sl-ppp (to be created)  
**Type**: task  
**Priority**: 3  
**Depends**: sl-ooo

**Acceptance Criteria**:
- [ ] Proc-macro code reviewed for best practices
- [ ] Error messages clear for invalid attributes?
- [ ] Code generation templates reviewed
- [ ] Should use darling for attribute parsing?
- [ ] All tests still passing

---

#### Task 5.5.4: Test Enhancement (TEST QUALITY)
**Beads**: sl-qqq (to be created)  
**Type**: task  
**Priority**: 3  
**Depends**: sl-ppp

**Acceptance Criteria**:
- [ ] Edge case attribute combinations tested
- [ ] Error message quality tested
- [ ] Complex nested type generation tested
- [ ] Generic type handling tested
- [ ] 5+ additional tests added

---

## Phase 5.6: Final Review & Documentation

**Duration**: 1 day  
**Focus**: Overall quality review and comprehensive documentation

### Tasks

#### Task 5.6.1: Comprehensive Testing
**Beads**: sl-rrr (to be created)  
**Type**: task  
**Priority**: 2  
**Depends**: Phases 5.1-5.4 complete (5.5 optional)

**Description**:
Run all tests across all phases and ensure quality.

**Acceptance Criteria**:
- [ ] cargo test --all passes
- [ ] cargo test --all --features metadata passes
- [ ] cargo test --all --features metadata-derive passes (if implemented)
- [ ] All phase tests passing
- [ ] No test warnings
- [ ] Test coverage reviewed
- [ ] Performance acceptable

---

#### Task 5.6.2: Documentation Review
**Beads**: sl-sss (to be created)  
**Type**: task  
**Priority**: 2  
**Depends**: sl-rrr

**Description**:
Review and enhance all Phase 5 documentation.

**Acceptance Criteria**:
- [ ] All public APIs documented
- [ ] Module-level documentation complete
- [ ] Examples in docs tested (cargo test --doc)
- [ ] PHASE5_METADATA_ARCHITECTURE.md updated with implementation notes
- [ ] README.md updated with Phase 5 features
- [ ] Migration guide created for metadata usage
- [ ] Turtle use case documented

---

#### Task 5.6.3: Final Architecture Review
**Beads**: sl-ttt (to be created)  
**Type**: task  
**Priority**: 2  
**Depends**: sl-sss

**Description**:
Final review of Phase 5 architecture and implementation quality.

**Acceptance Criteria**:
- [ ] Code review checklist complete
- [ ] API follows Rust guidelines
- [ ] No unsafe code introduced
- [ ] Error handling reviewed
- [ ] Performance characteristics documented
- [ ] Feature flags correctly configured
- [ ] Backward compatibility verified
- [ ] Integration with Phases 1-4 verified

---

#### Task 5.6.4: Clippy & Format
**Beads**: sl-uuu (to be created)  
**Type**: task  
**Priority**: 2  
**Depends**: sl-ttt

**Description**:
Final code quality check with cargo fmt and clippy.

**Acceptance Criteria**:
- [ ] cargo fmt --all run
- [ ] cargo clippy --all-targets --all-features -- -D warnings passes
- [ ] 0 clippy warnings introduced
- [ ] Code style consistent across Phase 5
- [ ] Ready for final review and merge

---

## Beads Task Dependencies

```
Epic: sl-wnc (Phase 5: Settings Metadata & Introspection)
│
├─ Phase 5.1: Core Metadata Types ✅ CREATED
│  ├─ sl-426: TDD Test Suite (RED)
│  ├─ sl-eh6: Implement Core Types (GREEN) [depends: sl-426]
│  ├─ sl-8wc: Architecture Review (REFACTOR) [depends: sl-eh6]
│  └─ sl-vnd: Test Enhancement [depends: sl-8wc]
│
├─ Phase 5.2: SettingsIntrospection Trait [depends: 5.1] ✅ CREATED
│  ├─ sl-45l: TDD Test Suite (RED)
│  ├─ sl-0d7: Implement Trait (GREEN) [depends: sl-45l]
│  ├─ sl-3qk: Architecture Review (REFACTOR) [depends: sl-0d7]
│  └─ sl-cjw: Test Enhancement [depends: sl-3qk]
│
├─ Phase 5.3: Validation Framework [depends: 5.2] ⏳ PENDING
│  ├─ TBD: TDD Test Suite (RED)
│  ├─ TBD: Implement Validation (GREEN)
│  ├─ TBD: Architecture Review (REFACTOR)
│  └─ TBD: Test Enhancement
│
├─ Phase 5.4: Integration & Examples [depends: 5.3] ⏳ PENDING
│  ├─ TBD: TDD Test Suite (RED)
│  ├─ TBD: Integration Implementation (GREEN)
│  ├─ TBD: Architecture Review (REFACTOR)
│  └─ TBD: Test Enhancement
│
├─ Phase 5.5: Proc-Macro (OPTIONAL) [depends: 5.4] ⏳ PENDING
│  ├─ TBD: TDD Test Suite (RED)
│  ├─ TBD: Implement Proc-Macro (GREEN)
│  ├─ TBD: Architecture Review (REFACTOR)
│  └─ TBD: Test Enhancement
│
└─ Phase 5.6: Final Review [depends: 5.4 or 5.5] ⏳ PENDING
   ├─ TBD: Comprehensive Testing
   ├─ TBD: Documentation Review
   ├─ TBD: Final Architecture Review
   └─ TBD: Clippy & Format
```

**Note**: Phases 5.3-5.6 tasks will be created as Phases 5.1-5.2 progress. Task IDs tracked in `history/PHASE5_TASK_IDS.md`.

---

## Test Count Estimate

- Phase 5.1: 20 tests (15 initial + 5 enhancements)
- Phase 5.2: 15 tests (10 initial + 5 enhancements)
- Phase 5.3: 25 tests (20 initial + 5 enhancements)
- Phase 5.4: 15 tests (10 initial + 5 enhancements)
- Phase 5.5: 20 tests (15 initial + 5 enhancements) [optional]
- **Total Core**: 75 tests
- **Total with Proc-Macro**: 95 tests

---

## Quality Gates

Before marking Phase 5 complete:

- [ ] All core tests passing (75 tests minimum)
- [ ] Optional proc-macro tests passing (if implemented)
- [ ] All existing tests still passing (backward compatibility)
- [ ] 0 clippy warnings introduced
- [ ] cargo fmt --all run
- [ ] Documentation complete with examples
- [ ] Integration with Phases 1-4 verified
- [ ] Turtle use case validated
- [ ] Feature flags configured correctly
- [ ] No unsafe code introduced

---

## Feature Flags Configuration

```toml
[features]
# Core metadata types (no new dependencies)
metadata = []

# Proc-macro for automatic generation
metadata-derive = ["metadata", "settings-loader-derive"]

# Full metadata support
full = ["metadata", "metadata-derive", "editor", "multi-scope"]
```

---

## Success Criteria Summary

Phase 5 is complete and approved when:

1. ✅ All core types implemented and tested
2. ✅ SettingsIntrospection trait functional
3. ✅ Validation framework working with clear errors
4. ✅ Integration examples demonstrate real-world usage
5. ✅ Optional proc-macro implemented (or deferred)
6. ✅ All tests passing (75+ tests)
7. ✅ Documentation complete with Turtle examples
8. ✅ 0 clippy warnings
9. ✅ 100% backward compatible
10. ✅ Ready for Phases 6-7 integration

---

**Created**: December 18, 2025  
**Author**: GitHub Copilot CLI Agent  
**Status**: Ready for Task Creation in Beads
