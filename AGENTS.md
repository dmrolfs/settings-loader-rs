# settings-loader-rs Agent Context

Project-specific agent workflow and guidelines. **Always reference global guidelines at** `~/Documents/dev/ai_agents/agents/AGENTS.md` first.

---

## 🌐 GLOBAL AGENT CONTEXT

**⚠️CRITICAL - READ FIRST**: This project follows global agent guidelines located at:

**📂 `.ai-context/`** (symlink to `/Users/rolfs/Documents/dev/ai_agents/agents/`)

### Required Reading at Session Start:

1. **`.ai-context/AGENTS.md`** - Core agent instructions and interaction patterns
2. **`.ai-context/CRITICAL.md`** - Critical constraints and requirements

### Lifecycle-Specific Guidelines:

- **`.ai-context/work-cycle/`** - Task management, work cycle patterns
- **`.ai-context/code/`** - Code quality, testing, review standards
- **`.ai-context/rust/`** - Rust-specific patterns and best practices
- **`.ai-context/vcs/`** - Git workflow and commit standards

**🎯 Action Required**: Agents should read relevant `.ai-context/` documents before:
- Starting any task
- Making architectural decisions
- Committing code
- Reviewing PRs

---

## Project Overview

**settings-loader-rs** v0.15.0 → v1.0.0 Evolution  
**Goal**: Transform from read-only configuration loader into comprehensive configuration management system  
**Status**: Phase 1 (Explicit Layering) ready for implementation approval  
**Timeline**: ~11 weeks (7 phases) from Phase 1 approval

### Key Principles for This Project

1. **Zero Breaking Changes**: All features strictly additive. Every new trait method has default implementation.
2. **Backward Compatibility Mandatory**: Existing code must continue working without modification.
3. **Feature Flags for Optional Deps**: Dependencies like `directories`, `toml_edit` gated behind feature flags.
4. **All Formats Preserved**: YAML, JSON, TOML, HJSON, RON supported throughout entire roadmap.
5. **TOML Comment Preservation**: Phase 4 feature using `toml_edit` - unique selling point vs. config-rs/figment.
6. **TDD Required**: All tests pre-written for Phase 1; implement against existing test suite.

---

## Before Starting Work

1. **Read Project Roadmap**:
   - `history/CONSOLIDATED_ROADMAP.md` - Master plan (all 7 phases)
   - `history/DESIGN.md` - Phase 1 detailed spec (if implementing Phase 1)
   - `history/IMPLEMENTATION_PLAN.md` - Step-by-step Phase 1 guide

2. **Understand Phase Context**:
   - Which phase is this work for? (1-7)
   - Review Phase section in CONSOLIDATED_ROADMAP.md
   - Check dependencies on earlier phases

3. **Memory Check**: `aim_search_nodes({query: "settings-loader", context: "work"})`

4. **Issue Triage**: `mcp__beads__ready` (if using beads for this project)

---

## TDD Workflow for This Project

### For Phase 1 (Explicit Layering) - CURRENT

1. **Tests Already Exist**: `tests/layer_builder_tests.rs` contains 25 comprehensive tests
   - Tests currently fail (red phase) - using stub implementations
   - Stub implementations at bottom of test file (lines ~650+)
   - All tests ready; no test writing needed

2. **Implementation Phase**:
   - Create `src/layer.rs` with real `ConfigLayer` enum and `LayerBuilder` struct
   - Replace stub implementations with real logic
   - Follow `IMPLEMENTATION_PLAN.md` phases 1-8:
     - Phase 1: Core Types (ConfigLayer, LayerBuilder)
     - Phase 2: Builder Methods
     - Phase 3: Build Implementation (layer loading logic)
     - Phase 4: File Format Detection
     - Phase 5: Integration with SettingsLoader
     - Phase 6: Enhance LoadingOptions Trait
     - Phase 7: Module Exports & Documentation
     - Phase 8: Backward Compatibility Validation

3. **Code Quality Gate** (before each commit):
   ```bash
   cargo fmt                    # Format
   cargo clippy                 # Lint (0 warnings required)
   cargo test                   # All tests must pass
   cargo check                  # Verify compilation
   ```

4. **Success Criteria**:
   - ✅ All 25 tests in `layer_builder_tests.rs` passing
   - ✅ All existing tests still pass (backward compatibility)
   - ✅ 0 clippy warnings
   - ✅ Documentation complete with examples
   - ✅ `MIGRATION_GUIDE.md` reviewed and validated

### For Future Phases (2-7)

1. **Design Phase**: May create new test file in `tests/` with comprehensive coverage
2. **Test Plan Review**: Present tests before implementation approval
3. **Implementation**: Follow TDD red → green → refactor cycle
4. **Validation**: Edge cases and integration scenarios
5. **Closure**: All tests passing, code review complete

---

## Phase-Specific Context

### Phase 1: Explicit Configuration Layering (READY NOW)

**Files**:
- Design: `history/DESIGN.md`
- Implementation: `history/IMPLEMENTATION_PLAN.md` (8 substeps)
- Tests: `tests/layer_builder_tests.rs` (25 tests, mostly stubs)
- Test Plan: `history/TEST_PLAN_SUMMARY.md`
- Migration: `history/MIGRATION_GUIDE.md`

**New Types** (to implement in `src/layer.rs`):
```rust
pub enum ConfigLayer {
    Path(PathBuf),
    EnvVar(String),
    EnvSearch { env: Environment, dirs: Vec<PathBuf> },
    Secrets(PathBuf),
    EnvVars { prefix: String, separator: String },
}

pub struct LayerBuilder { layers: Vec<ConfigLayer> }
```

**New Trait Method** (add to `LoadingOptions` in `src/lib.rs`):
```rust
fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
    builder  // Default: no explicit layers (backward compatible)
}
```

**Integration Point** (`src/settings_loader.rs`):
- Modify `SettingsLoader::load()` to check for explicit layers first
- Fall back to implicit layering if `build_layers()` not customized

**No New Dependencies**: Phase 1 uses only existing dependencies

**Testing**: All 25 tests pre-written; implementation makes them pass

---

### Phases 2-7: Quick Reference

| Phase | Feature | Timeline | Effort | Key API | Dependencies |
|-------|---------|----------|--------|---------|--------------|
| 2 | Env Var Customization | Week 2 | 2d | `env_prefix()`, `env_separator()` | None |
| 3 | Multi-Scope Paths | Week 3 | 4d | `ConfigScope`, `MultiScopeConfig` trait | `directories` |
| 4 | Config Editing | Weeks 4-5 | 6d | `LayerEditor` trait, `ConfigScope` | `toml_edit` |
| 5 | Metadata & Introspection | Weeks 6-7 | 6-8d | `SettingMetadata`, `ConfigSchema` | Optional proc-macro |
| 6 | Source Provenance | Weeks 8-9 | 8d | `SourceMap`, `SettingSource` enum | None (core refactor) |
| 7 | Schema Export | Weeks 10-11 | 4d | JSON Schema gen, HTML doc gen | None |

See `history/CONSOLIDATED_ROADMAP.md` for full details on each phase.

---

## Documentation Structure

### Strategic Documents (history/)
- `README.md` - Navigation guide for all documentation
- `EXECUTIVE_SUMMARY.md` - High-level overview + decision points
- `CONSOLIDATED_ROADMAP.md` - Master plan for all 7 phases
- `QUICK_REFERENCE.txt` - One-page quick reference

### Phase 1 Technical Documents (history/)
- `DESIGN.md` - Detailed Phase 1 specification
- `IMPLEMENTATION_PLAN.md` - Step-by-step implementation with 8 phases
- `TEST_PLAN_SUMMARY.md` - Test coverage overview
- `MIGRATION_GUIDE.md` - For users migrating to explicit layering

### Reference Documents (ref/)
- `architecture-proposal.md` - Original architectural vision
- `improvement-roadmap.md` - Gap analysis vs. current state
- `turtle-consolidation.md` - Spark-Turtle components to consolidate

### Test Files
- `tests/layer_builder_tests.rs` - 25 Phase 1 tests (ready to implement)

---

## Documentation Organization & Maintenance

**CRITICAL**: Maintain supporting documentation aligned with beads issue tracking for complete project context.

### Task-Supporting Documentation (./history/)

For each beads task (issue), maintain supporting markdown documents:
- **Location**: `history/` directory
- **Naming**: Prefix with beads issue ID: `{ISSUE_ID}_{DESCRIPTION}.md`
- **Examples**:
  - `PHASE1-001_layer_builder_implementation.md`
  - `PHASE1-001_test_results.md`
  - `PHASE1-001_blockers_and_decisions.md`

**Content Types**:
- Implementation progress and learnings
- Test results and coverage analysis
- Blockers discovered and resolutions
- Design adjustments and rationale
- Performance metrics and benchmarks
- Validation results

**Lifecycle**:
1. Create when task is started (copy/create from IMPLEMENTATION_PLAN.md skeleton)
2. **Maintain actively** - Update with findings, adjustments, learnings
3. Close with final summary when task completes
4. Reference in beads task notes (`beads note {ISSUE_ID} ...`)

### Large-Scope Documentation (./ref/)

For architectural, epic, and feature-level planning:
- **Location**: `ref/` directory
- **Naming**: Descriptive names without prefixes
- **Examples**:
  - `architecture-proposal.md` - Overall architecture vision
  - `improvement-roadmap.md` - Long-term improvement plan
  - `turtle-consolidation.md` - Specific feature consolidation
  - `phase2-env-customization-design.md` - Phase-level design
  - `roadmap-update-2025.md` - Major roadmap revisions

**Content Types**:
- Architectural decisions and rationale
- Epic/feature design specifications
- Long-term roadmap and vision
- Gap analysis and comparative studies
- Component consolidation plans

**Lifecycle**:
1. Create early in planning phase
2. Update as architecture evolves
3. Keep as reference for current and future phases
4. Archive old versions if major revisions needed

### Documentation Sync Workflow

**Before Starting Task Implementation**:
```bash
# Check if task has supporting doc in ./history
ls history/ | grep "^{ISSUE_ID}"

# If not found, create from IMPLEMENTATION_PLAN skeleton
cp history/IMPLEMENTATION_PLAN.md history/{ISSUE_ID}_{task_name}.md
```

**During Implementation**:
```bash
# Add section to task doc for each major finding
# Example in history/{ISSUE_ID}_{task_name}.md:

## Implementation Progress
### Phase 1-2: Core Types (COMPLETED)
- [x] ConfigLayer enum created with 5 variants
- [x] LayerBuilder struct with vec storage
- Tests passing: 6/25

### Phase 3: Build Implementation (IN_PROGRESS)
- [x] Path layer loading logic
- [ ] EnvVar layer resolution
- Blocker: Need path absolutization utility understanding
```

**Before Each Commit**:
```bash
# Update task doc with current status
# Reference related docs in ./ref if design evolved
# Commit both code AND updated ./{history|ref}/*.md files together
git add src/layer.rs history/{ISSUE_ID}_{task_name}.md
git commit -m "feat(phase1): implement LayerBuilder phase 3..."
```

**At Task Closure**:
```bash
# Final summary in task doc
# Link to PR/commits
# Note any tech debt or follow-ups for future phases
# Update main roadmap (history/CONSOLIDATED_ROADMAP.md) if needed
```

### Documentation Cross-References

In task documents (./history/), link to:
- Reference architecture docs: `See ref/architecture-proposal.md for context`
- Design specifications: `See history/DESIGN.md for full API`
- Other phase docs: `See history/{OTHER_ISSUE}_{name}.md for phase 2`

In reference documents (./ref/), link to:
- Roadmap: `See history/CONSOLIDATED_ROADMAP.md phases 1-7`
- Task tracking: `See beads task {ISSUE_ID} for implementation status`
- Design docs: `See history/DESIGN.md for Phase 1 specifics`

### Tools for Documentation Workflow

**View task docs for current beads task**:
```bash
# List all task docs
ls -la history/ | grep "^[A-Z]"

# View specific task doc
cat history/{ISSUE_ID}_{task_name}.md | head -50
```

**Search across documentation**:
```bash
# Find all mentions of a component
rg "LayerBuilder" history/ ref/

# Find TODOs across docs
rg "TODO|FIXME|XXX" history/ ref/

# Find task IDs in docs
rg "PHASE1-" history/
```

**Keep documentation in sync with code**:
- Do NOT commit code without updating supporting task docs
- Do NOT update roadmap/reference docs without updating AGENTS.md if workflow changes
- Task docs are living documents - update frequently during implementation

---

## Code Review & Approval Gates

**⚠️ CRITICAL RULE**: Agent MUST NEVER commit code or close tasks without explicit user review and approval.

- **NO ASSUMPTIONS**: Do not assume user will skip review. User ALWAYS performs code review.
- **NO AUTO-COMMITS**: Never commit code directly. Present work and wait for approval.
- **NO AUTO-CLOSURE**: Never close beads tasks without user approval.
- **ALWAYS STOP**: After implementation, present work for review. Wait for explicit approval before committing.

---

## Approval Gates for Phase 1

### Gate 1: Design & Test Review (COMPLETED)
- [x] Design specification complete (`history/DESIGN.md`)
- [x] 25 tests written (`tests/layer_builder_tests.rs`)
- [x] Implementation plan detailed (`history/IMPLEMENTATION_PLAN.md`)
- [x] Test plan documented (`history/TEST_PLAN_SUMMARY.md`)
- [x] Migration guide prepared (`history/MIGRATION_GUIDE.md`)
- **Status**: Awaiting **approval to proceed with implementation**

### Gate 2: Implementation Review (PENDING)
- [ ] Code review of implementation
- [ ] Verify all tests pass
- [ ] Check clippy warnings (0 required)
- [ ] Documentation complete and examples work
- **⚠️ ACTION**: **STOP after implementation. Present work. Await user approval before committing.**

### Gate 3: Integration Review (PENDING)
- [ ] Verify backward compatibility (all existing tests pass)
- [ ] Performance impact assessment
- [ ] Feature flag validation (if any)
- **⚠️ ACTION**: **STOP after integration testing. Present work. Await user approval before closing tasks.**

---

## Backward Compatibility Checklist

**Must verify before ANY commit**:

1. **Trait Methods**: New trait methods have default implementations
   ```rust
   pub trait LoadingOptions {
       // ... existing methods ...
       fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
           builder  // Default: no-op, backward compatible
       }
   }
   ```

2. **Existing Tests**: Run `cargo test --all` - ALL must pass
   - No modifications to existing test fixtures
   - Implicit layering behavior unchanged

3. **Enum Variants**: Use `#[non_exhaustive]` for extensibility
   ```rust
   #[non_exhaustive]
   pub enum SettingsError {
       // ... variants ...
   }
   ```

4. **Public API**: Only additive changes allowed
   - New types: ✅
   - New trait methods (with defaults): ✅
   - Modified existing methods: ❌
   - Removed types/methods: ❌

---

## Feature Flags Organization

Establish in `Cargo.toml` for full roadmap:

```toml
[features]
default = []

# Core layering (Phase 1)
layering = []

# Env var customization (Phase 2)
custom-env-format = []

# Multi-scope support (Phase 3)
multi-scope = ["directories"]

# Configuration editing (Phase 4)
editor = ["toml_edit"]

# Settings metadata (Phase 5)
schema = []
schema-derive = ["schema", "settings-loader-derive"]

# Source tracking (Phase 6)
provenance = []

# Documentation generation (Phase 7)
json-schema = ["schema"]
docs-gen = ["schema"]

# All features
full = [
    "layering", "custom-env-format", "multi-scope",
    "editor", "schema", "schema-derive",
    "provenance", "json-schema", "docs-gen"
]
```

For Phase 1: Keep existing feature flags unchanged, add no new ones.

---

## Key Tools & Patterns

### Code Search
```bash
# Find existing trait implementations
rg "impl SettingsLoader" src/

# Find all ConfigBuilder usages
rg "ConfigBuilder" src/

# Find environment variable handling
rg "env::var\|std::env" src/
```

### Testing
```bash
# Run all tests
cargo test --all

# Run Phase 1 tests specifically
cargo test layer_builder_tests

# Run with logging
RUST_LOG=debug cargo test layer_builder_tests -- --nocapture

# Watch mode during development
cargo watch -x "test layer_builder_tests"
```

### Code Quality
```bash
# Format all code
cargo fmt --all

# Check for clippy warnings (0 allowed)
cargo clippy --all-targets --all-features -- -D warnings

# Full check before commit
cargo fmt && cargo clippy && cargo test --all
```

---

## Commit Message Format

When committing Phase 1 work, include:
- What was implemented (which IMPLEMENTATION_PLAN phase)
- Test status (all X tests passing)
- Backward compatibility verified
- Clippy/formatting verified

Example:
```
feat(phase1): implement LayerBuilder core types and builder methods

- Implement ConfigLayer enum (Path, EnvVar, Secrets, EnvVars)
- Implement LayerBuilder struct with fluent API
- Add with_path, with_env_var, with_secrets, with_env_vars methods
- Tests: 6/25 passing (phase 1-2 substeps complete)
- Backward compatibility: verified with existing test suite
- Clippy: 0 warnings
```

---

## Git & GitHub Workflow

### Branch Strategy
- Work on feature branch: `feat/phase1-explicit-layering`
- Reference IMPLEMENTATION_PLAN phases in branch name if long-running

### Pull Request Template
Include in PR description:
- Which IMPLEMENTATION_PLAN phase(s) completed
- Test coverage status (X/25 tests passing)
- Backward compatibility verification
- Documentation updates

### Before Merge
- [ ] All tests passing (`cargo test --all`)
- [ ] 0 clippy warnings
- [ ] Code formatted (`cargo fmt`)
- [ ] PR reviewed and approved
- [ ] Backward compatibility confirmed

---

## Success Definition

**Phase 1 is "done" when**:

- ✅ All 25 tests in `layer_builder_tests.rs` pass
- ✅ All existing tests still pass (backward compatibility)
- ✅ 0 clippy warnings
- ✅ Documentation complete with examples
- ✅ Code review approved
- ✅ Ready for v0.16.0 release

---

## Questions & Escalation

| Question | Reference |
|----------|-----------|
| "How does explicit layering work?" | `history/DESIGN.md` Section "Type Definitions" |
| "What are the implementation steps?" | `history/IMPLEMENTATION_PLAN.md` Phases 1-8 |
| "How many tests and what do they cover?" | `history/TEST_PLAN_SUMMARY.md` + `tests/layer_builder_tests.rs` |
| "How do I migrate existing code?" | `history/MIGRATION_GUIDE.md` |
| "What about future phases?" | `history/CONSOLIDATED_ROADMAP.md` Phases 2-7 |
| "Why this design?" | `history/EXECUTIVE_SUMMARY.md` Section "Key Design Decisions" |
| "How does this help Turtle?" | `ref/turtle-consolidation.md` |

---

## Immediate Next Steps

1. **User Decision**: Approve Phase 1 for implementation?
2. **If approved**:
   - [ ] Create feature branch: `feat/phase1-explicit-layering`
   - [ ] Create `src/layer.rs` with type stubs
   - [ ] Follow IMPLEMENTATION_PLAN.md phases 1-8
   - [ ] Make 25 tests pass (green phase)
   - [ ] Request code review approval (Gate 2)
   - [ ] Request integration approval (Gate 3)
   - [ ] Prepare release notes for v0.16.0

3. **If questions**:
   - Consult `history/README.md` for documentation navigation
   - Review relevant phase in `history/CONSOLIDATED_ROADMAP.md`
   - Check `history/EXECUTIVE_SUMMARY.md` for high-level context

---

**Last Updated**: 2025-12-15  
**Status**: Awaiting Phase 1 Implementation Approval
