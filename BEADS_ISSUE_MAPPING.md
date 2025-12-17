# Beads Issue Mapping - settings-loader-rs v0.22.0

Complete tracking of beads issue IDs for all phases with dependency relationships.

---

## Epic Issues

| Epic | Phase | Status | Total Subtasks | Tests |
|------|-------|--------|-----------------|-------|
| sl-h8h | Phase 1: Explicit Layering | ✅ COMPLETE | 8 | 27/27 ✅ |
| sl-365 | Phase 2: Env Customization | ✅ COMPLETE | 3 | 12/12 ✅ |
| sl-ozp | Phase 3: Multi-Scope | 📋 READY | 4 | 14/14 (tests pre-written) |

---

## Phase 1: Explicit Configuration Layering API

**Epic**: sl-h8h  
**Status**: ✅ COMPLETE  
**Tests**: 27/27 passing  

### Subtask Chain
```
sl-bru (PHASE1.1: Core Types)
  ↓ blocks
sl-bv4 (PHASE1.2: Builder Methods)
  ↓ blocks
sl-nz1 (PHASE1.3: Build Implementation)
  ↓ blocks
sl-5h9 (PHASE1.4: File Format Detection)
  ↓ blocks
sl-apw (PHASE1.5: SettingsLoader Integration)
  ↓ blocks
sl-mvm (PHASE1.6: LoadingOptions Enhancement)
  ↓ blocks
sl-uda (PHASE1.7: Documentation & Exports)
  ↓ blocks
sl-6pk (PHASE1.8: Validation & Quality)
```

### Details

| Issue | Title | Type | Status | Design |
|-------|-------|------|--------|--------|
| sl-bru | Core Types - ConfigLayer enum + LayerBuilder struct | Task | ✅ Closed | src/layer.rs - 5 enum variants, Vec storage |
| sl-bv4 | Builder Methods - with_path, with_env_var, etc. | Task | ✅ Closed | Fluent API implementation |
| sl-nz1 | Build Implementation - Layer loading logic | Task | ✅ Closed | Path absolutization, env var resolution, layer precedence |
| sl-5h9 | File Format Detection - Leverage config crate | Task | ✅ Closed | YAML, JSON, TOML, HJSON, RON support |
| sl-apw | SettingsLoader Integration | Task | ✅ Closed | Explicit layers with fallback to implicit |
| sl-mvm | LoadingOptions Enhancement - build_layers() | Task | ✅ Closed | Default trait method, backward compatible |
| sl-uda | Documentation & Exports | Task | ✅ Closed | Module docs, examples, pub exports |
| sl-6pk | Validation & Quality | Task | ✅ Closed | All tests pass, clippy clean, fmt applied |

---

## Phase 2: Environment Variable Customization

**Epic**: sl-365  
**Status**: ✅ COMPLETE  
**Tests**: 12/12 passing  

### Subtask Chain
```
sl-dtc (PHASE2.1: Test Suite)
  ↓ blocks
sl-dp8 (PHASE2.2: Trait Enhancement)
  ↓ blocks
sl-xkr (PHASE2.3: Integration & Validation)
```

### Details

| Issue | Title | Type | Status | Design |
|-------|-------|------|--------|--------|
| sl-dtc | Test Suite - Environment Variable Customization | Task | ✅ Closed | tests/phase2_env_customization_tests.rs - 12 tests |
| sl-dp8 | Trait Enhancement - env_prefix() and env_separator() | Task | ✅ Closed | Static methods on LoadingOptions, default "APP" and "__" |
| sl-xkr | Integration & Validation - LayerBuilder + LoadingOptions | Task | ✅ Closed | Verify custom conventions work end-to-end |

---

## Phase 3: Multi-Scope Configuration Support

**Epic**: sl-ozp  
**Status**: 📋 READY FOR IMPLEMENTATION  
**Tests**: 14 pre-written (ready to compile)  

### Subtask Chain
```
sl-x7d (PHASE3.1: Test Suite)
  ↓ blocks
sl-wcu (PHASE3.2: ConfigScope Enum & Find Logic)
  ↓ blocks
sl-4ug (PHASE3.3: MultiScopeConfig Trait)
  ↓ blocks
sl-evw (PHASE3.4: LayerBuilder Integration)
```

### Details

| Issue | Title | Type | Status | Design |
|-------|-------|------|--------|--------|
| sl-x7d | Test Suite - Multi-Scope Configuration | Task | 📋 Open | tests/phase3_multi_scope_tests.rs - 14 tests (pre-written) |
| sl-wcu | ConfigScope Enum & Config File Discovery | Task | 📋 Open | src/scope.rs - 4 variants, find_config_in() utility |
| sl-4ug | MultiScopeConfig Trait - Path Resolution | Task | 📋 Open | src/loading_options.rs - trait with directories crate |
| sl-evw | LayerBuilder Integration - with_scopes() | Task | 📋 Open | src/layer.rs - convenience method for multi-scope |

---

## Phase 4-7 (Planned)

Future epics will follow same pattern:
- **Phase 4** (sl-m17): Configuration Editing & Writing - 4 subtasks
- **Phase 5** (sl-wnc): Settings Metadata & Introspection - 4 subtasks  
- **Phase 6** (sl-1w8): Source Provenance & Tracking - 3 subtasks
- **Phase 7** (sl-0j8): Schema & Documentation Generation - 3 subtasks

---

## How to Use This Mapping

### Starting a Phase
1. Find epic ID (sl-ozp for Phase 3, etc.)
2. Check dependencies: can't start until previous phase epic subtasks closed
3. Work through subtask chain in dependency order

### During Implementation
```bash
# Check status of current issue
beads show sl-x7d

# Update status when starting work
beads update sl-x7d --status in_progress

# When complete
beads update sl-x7d --status closed
# Next issue (sl-wcu) is now unblocked
```

### Before Committing
```bash
# Run test suite for current phase
cargo test --test phase3_multi_scope_tests

# Quality gates
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all  # Verify backward compat
```

---

## Issue Dependencies Summary

### Phase 1 Dependencies
```
sl-bru
  ↓ (sl-bv4 depends on sl-bru)
sl-bv4
  ↓ (sl-nz1 depends on sl-bv4)
sl-nz1
  ↓ (sl-5h9 depends on sl-nz1)
sl-5h9
  ↓ (sl-apw depends on sl-5h9)
sl-apw
  ↓ (sl-mvm depends on sl-apw)
sl-mvm
  ↓ (sl-uda depends on sl-mvm)
sl-uda
  ↓ (sl-6pk depends on sl-uda)
sl-6pk (final validation)
```

### Phase 2 Dependencies
```
sl-dtc
  ↓ (sl-dp8 depends on sl-dtc)
sl-dp8
  ↓ (sl-xkr depends on sl-dp8)
sl-xkr (final validation)
```

### Phase 3 Dependencies (READY NOW)
```
sl-x7d
  ↓ (sl-wcu depends on sl-x7d)
sl-wcu
  ↓ (sl-4ug depends on sl-wcu)
sl-4ug
  ↓ (sl-evw depends on sl-4ug)
sl-evw (final validation)
```

---

## Quick Reference

### All Closed Issues (Phase 1-2)
- sl-bru, sl-bv4, sl-nz1, sl-5h9, sl-apw, sl-mvm, sl-uda, sl-6pk (Phase 1)
- sl-dtc, sl-dp8, sl-xkr (Phase 2)

### All Open Issues (Phase 3+)
- sl-x7d, sl-wcu, sl-4ug, sl-evw (Phase 3)
- sl-m17, sl-wnc, sl-1w8, sl-0j8 (Phases 4-7 epics)

### Test File to Issue Mapping
- tests/layer_builder_tests.rs → sl-bru through sl-6pk (Phase 1)
- tests/phase2_env_customization_tests.rs → sl-dtc through sl-xkr (Phase 2)
- tests/phase3_multi_scope_tests.rs → sl-x7d through sl-evw (Phase 3)

---

**Last Updated**: Dec 16, 2025  
**Phase 3 Ready**: ✅  
**Next Action**: Start sl-x7d verification
