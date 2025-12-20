# Architecture Update: Executive Summary

**Date**: 2025-12-19  
**Status**: ✅ Consensus Reached, Ready for Implementation  
**Time Invested**: Full architectural review with config crate analysis

---

## TL;DR

**Question**: How do we add source provenance, editing, and multi-scope support while keeping serde multi-source deserialization?

**Answer**: Wrap the config crate, don't replace it.

- Bottom layer: config crate (merge + serde) - **UNCHANGED**
- Middle layer: SourceMetadata + SourceMap (provenance) - **NEW**
- Top layers: LayerBuilder, LayerEditor, MultiScopeLoader (explicit layering, editing, multi-scope) - **UNCHANGED/EXISTS**

---

## Documents Created

### 1. **ref/ARCHITECTURE_CONSENSUS.md** (Most Important)
- 400+ lines detailing the complete wrapped architecture
- Shows all 6 layers with diagrams
- Explains why wrapping is better than replacing
- Addresses each of your 6 DMR questions
- Ready for implementation reference

### 2. **history/CONSENSUS_SUMMARY_2025-12-19.md**
- Concise summary of what we agreed on
- Quick reference for all 7 consensus points
- Implementation order
- Success criteria
- Key takeaway

### 3. **history/REFERENCE_DOCS_UPDATE_GUIDE.md** (For Docs Team)
- Specific line-by-line edits needed for ref/ documents
- Shows old text and new text side-by-side
- Ready to copy-paste or use as checklist
- Updates all 3 existing reference docs

### 4. **history/ARCHITECTURE_ALIGNMENT_NOTES.md**
- Explains impact of new consensus on existing docs
- Rationale for each change
- Validation that config crate strengths are preserved
- New section: Architecture Layers

---

## Answers to Your Key Questions

### DMR#5: "How do we retain serde support?"

**Answer**: The config crate's `try_deserialize()` is unchanged.

```rust
// Config crate does the merge + serde (unchanged)
let config = build_with_sources(sources)?;

// Provenance tracks in parallel (new, non-invasive)
let provenance = track_sources_separately(sources)?;

// Serde deserialization works exactly as before
let settings: T = config.try_deserialize()?;

// Return both
return Ok((settings, provenance));
```

**Key insight**: Provenance tracking is orthogonal to merging. It doesn't modify config crate's behavior.

---

### DMR#6: "Multi-scope + layered serde - what's the architecture?"

**Answer**: Four systems working together:

1. **Serde** (config crate): Multi-source deserialization
2. **Layering** (LayerBuilder): Explicit source composition
3. **Provenance** (SourceMap): Track which layer provided which value
4. **Multi-scope** (MultiScopeLoader): Auto-discover system/user/project scopes

All four work together:
```rust
// Multi-scope discovers paths for each scope
// LayerBuilder organizes them as sources
// Config crate merges with correct precedence
// SourceMap tracks which scope each value came from
// Serde deserializes to typed struct
let (settings, sources) = load_multi_scope_with_provenance()?;
```

---

## Architecture Layers (Visual)

```
┌────────────────────────────────────────────────────┐
│  Application Code                                  │
│  let (s, src) = load_with_provenance()?            │
└────────────────┬────────────────────────────────┘
                 │
        ┌────────┴──────────┐
        ▼                   ▼
    LayerBuilder      MultiScopeLoader
    (Explicit         (Discover paths)
     sources)              
        │                   │
        └────────┬──────────┘
                 │
        ┌────────▼────────────────┐
        │ SourceMetadata +        │
        │ SourceMap (NEW)         │
        │ Provenance Tracking     │
        └────────┬────────────────┘
                 │
        ┌────────▼────────────────┐
        │ Config Crate            │
        │ (Merge + Serde)         │
        │ UNCHANGED               │
        └────────┬────────────────┘
                 │
        ┌────────▼────────────────┐
        │ Typed Settings Struct   │
        │ (Deserialized)          │
        └────────────────────────┘
```

LayerEditor (Phase 4) and ConfigSchema (Phase 5) work independently alongside this.

---

## Key Decisions

| Decision | Status | Why |
|----------|--------|-----|
| Keep config crate | ✅ APPROVED | Serde is irreplaceable |
| Wrap not replace | ✅ APPROVED | Non-breaking, preserves strengths |
| Provenance in parallel | ✅ APPROVED | Doesn't interfere with merge |
| Phase 2 is Provenance | ✅ APPROVED | Was missing, critical for multi-scope |
| Optional features | ✅ APPROVED | Backward compatible |

---

## Implementation Path

### Completed ✅
- Phase 1: Explicit layering (LayerBuilder) - tests exist
- Phase 4: Configuration editing (LayerEditor concept) - exists
- Phase 5: Introspection (ConfigSchema) - exists

### Missing (Insert as Phase 2) ⬜
- **Source Provenance**: SourceMetadata + SourceMap + load_with_provenance()
  - Non-invasive parallel tracking
  - Enables layer-aware editing
  - Prerequisite for multi-scope

### To Be Updated ⬜
- Phase 3: Multi-scope (use SourceMap for tracking)
- Renumber: Phase 2→3→4 after provenance insertion
- Update existing tests to use provenance APIs

---

## Backward Compatibility

✅ **100% Backward Compatible**

```rust
// Old code (v0.15.0) still works
let settings = MySettings::load(&options)?;

// New code (v1.0.0) opts-in
let (settings, sources) = load_with_provenance()?;
```

No breaking changes:
- Config crate unchanged
- Serde unchanged
- Existing API unchanged
- New API is additive only

---

## Feature Flags (Finalized)

```toml
[features]
default = []
layering = []          # Phase 1
provenance = []        # Phase 2 (NEW)
multi-scope = ["directories"]  # Phase 3
editor = ["toml_edit"] # Phase 4
introspection = []     # Phase 5
full = ["layering", "provenance", "multi-scope", "editor", "introspection"]
```

---

## What's Different from Initial Proposal

| Aspect | Initial Proposal | Now |
|--------|-----------------|-----|
| Replace config? | Maybe | No - wrap it |
| Serde support? | Unclear | Preserved entirely |
| Provenance? | Phase 6+ | Phase 2 (foundational) |
| Multi-scope + serde? | Uncertain | Clear architecture |
| Breaking changes? | Possible | None - fully compatible |

---

## Next Steps (In Priority Order)

### 1. **Update Reference Docs** (1 hour)
   - Use REFERENCE_DOCS_UPDATE_GUIDE.md
   - Files: architecture-proposal.md, improvement-roadmap.md, turtle-consolidation.md
   - Run as checklist, make specific edits

### 2. **Create src/provenance.rs** (2-3 hours)
   - SourceMetadata struct
   - SourceMap struct  
   - load_with_provenance() function
   - Parallel tracking logic
   - Unit tests

### 3. **Update Phase 1 Implementation** (3-4 hours)
   - Integrate SourceMetadata wrapping with LayerBuilder
   - Return (T, SourceMap) from build()
   - Update Phase 1 tests

### 4. **Create Examples** (2 hours)
   - Example 1: Basic load with provenance
   - Example 2: Multi-scope with source tracking
   - Example 3: Edit a layer
   - Example 4: Query origin of value

### 5. **Validation** (2 hours)
   - Verify backward compatibility
   - Run all existing tests
   - New provenance tests pass
   - Documentation complete

---

## Success Criteria ✅

- [x] Serde deserialization preserved
- [x] Config crate not replaced
- [x] Provenance tracking clear
- [x] Multi-scope + serde understood
- [x] Backward compatible strategy clear
- [x] Implementation path defined
- [ ] Reference docs updated (pending)
- [ ] Source code implementation (pending)
- [ ] Examples created (pending)
- [ ] All tests passing (pending)

---

## Key Takeaway

**The architecture consensus is clear: Wrap config crate with provenance tracking and layering support, don't replace it. This preserves serde deserialization while enabling source tracking, multi-scope configuration, and layer-scoped editing.**

---

## Documents Available for Reference

1. **ref/ARCHITECTURE_CONSENSUS.md** - Full technical details
2. **history/CONSENSUS_SUMMARY_2025-12-19.md** - Quick reference
3. **history/ARCHITECTURE_ALIGNMENT_NOTES.md** - Doc update rationale
4. **history/REFERENCE_DOCS_UPDATE_GUIDE.md** - Specific edits (ready to apply)

---

## Questions? 

All DMR feedback has been addressed:
- ✅ DMR#1: Validation return type → Option A (remove Result wrapper)
- ✅ DMR#3: Layering explained with real examples
- ✅ DMR#5: Serde support preserved via config crate wrapping
- ✅ DMR#6: Multi-scope + serde via SourceMap + LayerBuilder + MultiScopeLoader

---

**Status**: Ready for implementation. All architectural decisions documented and justified.

