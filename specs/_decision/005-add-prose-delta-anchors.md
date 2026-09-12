# Decisions: add-prose-delta-anchors

## ADR: Anchor is the first non-empty line of a delta block

**ID:** anchor-first-nonempty-line
**Plan:** add-prose-delta-anchors
**Status:** Accepted

### Context

A delta block's anchor must identify the section it targets without ambiguity. A whole-block scan for a heading lets a heading-like line inside the prose hijack the anchor.

### Decision

`extract_anchor` reads only the first non-empty line of a delta block. `### Scenario: <name>` and `## Background` match exactly. `# Feature` matches by prefix, matching the leniency of the spec parser at `src/validate/parser.rs:189-190`.

### Options Considered

| Option | Verdict |
|--------|---------|
| First non-empty line, exact or prefix match | ✓ Chosen — every existing scenario block already starts with its `### Scenario:` heading, so current behavior is preserved |
| Scan every line of the block for the first heading | ✗ Rejected — cannot distinguish the block's own heading from a heading quoted inside the prose |

### Consequences

Anchor detection is unambiguous and needs no new syntax. A delta author places the target heading on the first line of the block.

## ADR: A prose delta wraps the skeleton heading in place

**ID:** prose-delta-wraps-skeleton-heading
**Plan:** add-prose-delta-anchors
**Status:** Accepted

### Context

Every delta file already carries a description, a `## Background` section, and a `## Scenarios` section, because `validate::run` requires them. No existing convention lets a delta change Background or description text.

### Decision

A Background or description delta places the marker directly around the `## Background` or `# Feature: <name>` heading the delta file already carries. No duplicate heading and no new marker syntax.

### Options Considered

| Option | Verdict |
|--------|---------|
| Wrap the existing skeleton heading in place | ✓ Chosen — no new syntax to learn or parse |
| New marker attribute, for example `DELTA:CHANGED:background` | ✗ Rejected — adds syntax with no matching benefit |
| Duplicate heading copied into the Scenarios section | ✗ Rejected — diverges from the skeleton the file already has |

### Consequences

Authors reuse `DELTA:CHANGED` for prose exactly as for scenarios. The parser requires no change.

## ADR: One shared legality matrix owned by check_anchor_rules

**ID:** shared-anchor-legality-matrix
**Plan:** add-prose-delta-anchors
**Status:** Accepted

### Context

`speq plan validate` and `speq record` both reject illegal `(DeltaKind, DeltaAnchor)` combinations. Two separate implementations of the same rule set can drift apart.

### Decision

`record::check_anchor_rules(&[DeltaBlock]) -> Vec<String>` is the single owner of the legality matrix. Both commands call it. `src/plan.rs` names no heading string and derives no anchor.

### Options Considered

| Option | Verdict |
|--------|---------|
| Single shared function called by both gates | ✓ Chosen — the two gates agree by construction |
| Re-implement the matrix in `src/plan.rs` | ✗ Rejected — two copies of one decision drift apart |

### Consequences

A future rule change touches one function. The validate and record gates cannot disagree.

## ADR: plan validate checks the matrix only, never anchor presence

**ID:** plan-validate-skips-anchor-presence
**Plan:** add-prose-delta-anchors
**Status:** Accepted

### Context

Whether an anchor exists in the target spec can be known only by reading that spec, and another plan may rewrite the target spec before this plan reaches record.

### Decision

`validate_delta_anchors` applies the target-independent rules only. The record command checks anchor presence in the target spec. A delta file that adds a scenario with `DELTA:NEW` and edits it with `DELTA:CHANGED` on the same anchor is an error under this same rule.

### Options Considered

| Option | Verdict |
|--------|---------|
| Check target-independent rules only at validate time | ✓ Chosen — keeps `check_anchor_rules` free of I/O and usable by both gates |
| Also read the target spec at validate time | ✗ Rejected — reads a snapshot that can go stale before record runs |

### Consequences

`speq plan validate` cannot catch a missing-anchor error that only `speq record` reports. `cli/plan-validate` documents this boundary.

## ADR: A prose-realignment note in the plan notes directory, distinct from the planning note

**ID:** prose-realignment-note-in-plan-notes
**Plan:** add-prose-delta-anchors
**Status:** Accepted

### Context

A Background or description change needs an audit trail separate from the day-to-day planning scratch file.

### Decision

`record_plan` writes `specs/_plans/<plan-name>/notes/prose-realignment.md` before the archive step, so the existing `fs::rename` carries it into `specs/_recorded/NNN-<plan-name>/notes/`. It writes the file only when a Background or description section changed.

### Options Considered

| Option | Verdict |
|--------|---------|
| Write to the plan's `notes/` directory | ✓ Chosen — colocated with the plan, archived by the existing rename, no extra code |
| Rely on `git log -p` against the permanent spec | ✗ Rejected — leaves no explicit record of the before and after text |
| Write to the permanent spec directory | ✗ Rejected — mixes plan evidence into the permanent library |

### Consequences

The note is local evidence, since `specs/_recorded/` is gitignored in this repository. It does not become committed history.

## ADR: Recording is all or nothing across the plan

**ID:** record-all-or-nothing-across-plan
**Plan:** add-prose-delta-anchors
**Status:** Accepted

### Context

Turning three previously silent merge cases into errors is unsafe unless the write set is atomic. Otherwise a later error leaves an earlier permanent spec already overwritten, and a re-run after the fix would double-apply or fail.

### Decision

`record_plan` runs in three phases. It checks every delta spec's anchor rules and merges every delta into an in-memory `Vec<(PathBuf, String)>`, then calls `fs::write` only after every check and merge succeeds.

### Options Considered

| Option | Verdict |
|--------|---------|
| Three-phase check, merge, then write | ✓ Chosen — a failing delta leaves every permanent spec unchanged, so a re-run applies each delta exactly once |
| Write each merged spec inside the loop, stop on the first error | ✗ Rejected — leaves earlier specs already overwritten when a later delta fails |

### Consequences

A rejected plan can be fixed and re-recorded safely. The guarantee stops before the write, note-write, and archive phases. A file-system error in those phases is outside the guarantee.

## ADR: One block per anchor per delta file

**ID:** one-block-per-anchor-per-delta-file
**Plan:** add-prose-delta-anchors
**Status:** Accepted

### Context

Two blocks in one delta file that share a `(kind, anchor)` pair, or two blocks of different kinds on the same anchor, produce a merged result that depends on the blocks' order in the file.

### Decision

Two blocks of one delta file sharing a recognized anchor are an error, whatever the marker kinds. The merge matrix carries one row for each case, and `check_anchor_rules` owns the rule.

### Options Considered

| Option | Verdict |
|--------|---------|
| Reject any second block on a recognized anchor | ✓ Chosen — the merged result no longer depends on block order, since the author writes one block carrying the final text |
| Merge a NEW block and a CHANGED block on one anchor in a fixed order | ✗ Rejected — the outcome silently depends on which block appears first in the file |

### Consequences

An author who wants a scenario added and then edited in the same delta file must instead write one block with the final text.

## ADR: record_plan is record's call site for the legality matrix

**ID:** record-plan-calls-shared-legality-matrix
**Plan:** add-prose-delta-anchors
**Status:** Accepted

### Context

Naming `check_anchor_rules` the single owner of the legality matrix has no effect unless the record path calls it, instead of re-deriving the same rules inside the merge function.

### Decision

`merge_delta_tracked` implements only the row behaviors and the target-dependent anchor-not-found error, and does not re-derive legality. `record_plan` calls `check_anchor_rules` across the parsed blocks of every delta spec before the write phase, and maps each message to `RecordError::InvalidDeltaAnchor`.

### Options Considered

| Option | Verdict |
|--------|---------|
| `record_plan` calls `check_anchor_rules` before merging | ✓ Chosen — the two gates cannot drift apart |
| Build the legality matrix a second time inside `merge_delta_tracked` | ✗ Rejected — reproduces the two-copies-drift-apart failure the shared-matrix decision exists to prevent |

### Consequences

`merge_delta_tracked` stays a pure application of one row's behavior. `record_plan` is the documented call site for the shared rule set.

## ADR: Record skips anchor checks for a new feature spec

**ID:** record-skips-anchor-checks-new-feature
**Plan:** add-prose-delta-anchors
**Status:** Accepted

### Context

A new-feature delta wraps `## Background` in a `DELTA:NEW` block, which `check_anchor_rules` would otherwise reject. Skipping the check only inside `merge_delta_tracked`, reached solely for an existing target, is not enough once `check_anchor_rules` runs earlier in `record_plan`.

### Decision

`record_plan` skips `check_anchor_rules` for a delta spec whose target `<domain>/<feature>/spec.md` is absent under `specs/`, using the same condition as the validate-side exemption.

### Options Considered

| Option | Verdict |
|--------|---------|
| Skip anchor checks when the target spec does not exist | ✓ Chosen — keeps `speq plan validate` and `speq record` in agreement for new-feature deltas |
| Apply the legality matrix unconditionally | ✗ Rejected — rejects the standard new-feature delta shape and breaks the non-goal of leaving new-feature recording unchanged |

### Consequences

A new-feature delta records the same way it did before this plan. Both gates document the exemption identically.

## ADR: A CHANGED prose block deletes every line it omits

**ID:** changed-prose-block-deletes-omitted-lines
**Plan:** add-prose-delta-anchors
**Status:** Accepted

### Context

A `DELTA:CHANGED` block on `## Background` or `# Feature: <name>` replaces the whole section. A Background section accumulates bullets from many plans over time, so an author who omits a bullet in a new delta silently deletes it, a higher risk than for a self-contained scenario.

### Decision

State the deletion semantics in the plan's impact section and in the delta-authoring guidance: any line the block omits is deleted from the permanent spec, so an author copies the current section before editing it.

### Options Considered

| Option | Verdict |
|--------|---------|
| Document the full-replace, copy-first practice | ✓ Chosen — makes the risk visible before an author writes a partial Background block |
| Leave the omission risk undocumented | ✗ Rejected — a plan can silently drop unrelated Background bullets it did not intend to touch |

### Consequences

Delta authors read the full current section before writing a CHANGED prose block. The authoring template carries this instruction.

## ADR: merge_delta_tracked takes the feature path as an argument

**ID:** merge-delta-tracked-takes-feature-argument
**Plan:** add-prose-delta-anchors
**Status:** Accepted

### Context

`merge_delta_tracked` returned a `ProseChange` with an empty `.feature` field that `record_plan` patched afterward, so no single piece of code owned the invariant that every `ProseChange` names a feature.

### Decision

`merge_delta_tracked(feature: &str, existing: &str, delta: &str)` receives the `<domain>/<feature>` path and populates `ProseChange.feature` itself, so every returned value is valid at construction. `merge_delta` passes an empty feature path and discards the list.

### Options Considered

| Option | Verdict |
|--------|---------|
| Producer receives and sets the feature path | ✓ Chosen — every returned `ProseChange` is valid where it is constructed |
| Consumer patches the field after the call | ✗ Rejected — the producer emits a struct it knows is incomplete and shifts the invariant onto the caller |

### Consequences

`merge_delta`'s public signature and its four existing tests stay unaffected, since it remains a thin wrapper.
