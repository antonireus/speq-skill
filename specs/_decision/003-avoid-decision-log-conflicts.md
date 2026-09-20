# Decisions: avoid-decision-log-conflicts

## ADR: Per-plan fragment files replace the shared decision log

**ID:** per-plan-decision-fragments
**Plan:** avoid-decision-log-conflicts
**Status:** Accepted

### Context

Parallel plans routinely hit merge conflicts in `specs/decision-log.md`. `recorder-agent` computes the next ADR number by counting existing `## ADR-` headings, appends the block at end-of-file, and the validator enforces gap-free numbering starting at ADR-001. Two records therefore collide three ways — same next number, same insertion point, and a no-gaps rule that forces both into one sequence.

### Decision

Store one committed ADR fragment per plan at `specs/_decision/NNN-<plan-name>.md`; delete `specs/decision-log.md`. Different plans touch different files, so git never conflicts on the text.

### Options Considered

| Option | Verdict |
|--------|---------|
| Per-plan fragment files | ✓ Chosen — only disjoint file paths remove the git conflict structurally |
| Keep one shared file with smarter merge or a write lock | ✗ Rejected — a shared append-only file still forces coordination on a single artifact |

### Consequences

Parallel plans write disjoint files, so git never contends on the text. Reading the full decision history requires assembling fragments rather than reading one file. `speq decision-log show` assembles the fragments to stdout and no merged file is committed. The `NNN-` prefix is a record-time count; two parallel plans can share one, and `show` orders by `(prefix, filename)` to stay deterministic.

## ADR: Stable slug identity replaces sequential ADR numbers

**ID:** slug-identity-not-sequential-numbers
**Plan:** avoid-decision-log-conflicts
**Status:** Accepted

### Context

Sequential `ADR-NNN` numbering requires a single shared counter. Parallel plans promoting ADRs independently cannot agree on the next number without coordination, which reintroduces the conflict that per-plan fragment files were meant to remove.

### Decision

Identify each ADR by an explicit kebab-case `**ID:**` slug, unique across all fragments, rejecting sequential `ADR-NNN` numbering because it forces renumbering and global coordination across parallel plans. `Supersedes:` and `Status: Superseded by <slug>` reference slugs instead of numbers. Delete the sequential-number validator. Cross-references survive independent promotion without renumbering.
