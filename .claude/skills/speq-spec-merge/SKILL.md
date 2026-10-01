---
name: speq-spec-merge
description: Recording procedure — library-threshold pre-check, ADR-promotion field mapping, and the `speq record` merge-and-archive step. Triggered by recorder-agent.
---

# Spec Merge

## Load Plan Context

```
Read: specs/_plans/<plan-name>/plan.md
List: specs/_plans/<plan-name>/**/spec.md
```

Run `speq feature list` to see the current permanent spec library.

## Check Library Thresholds

Before recording, compute each target's size after the merge. Scenarios per spec: the target spec's `### Scenario:` count, plus the plan's `DELTA:NEW` blocks, minus its `DELTA:REMOVED` blocks (a new feature counts its own scenarios). Features per domain: the domain's feature directories plus the plan's new features.

| Metric | Threshold | Action |
|--------|-----------|--------|
| Scenarios per spec | >10 | Return to orchestrator for user decision |
| Domain features | >8 | Return to orchestrator for user decision |

**Never assume:** library reorganization is a user decision. Return a concrete question to the orchestrator, and do not record.

## Promote ADRs to Permanent Decision Log

Read `specs/_plans/<plan-name>/decision-log.md` (if it exists).

For each entry where `Promotes to ADR: yes`:

1. Convert to ADR format using `/speq-plan`'s `references/decision-log-permanent-template.md`:
   - **Title** — from the decision entry heading
   - **ID** — a kebab-case slug derived from the title; MUST be unique across every file in `specs/_decision/`
   - **Plan** — `<plan-name>`
   - **Status** — `Accepted`
   - **Supersedes** (optional) — if the entry names an earlier decision it replaces, set this to that decision's existing ADR slug
   - **Context** — synthesized from the entry's Rationale + Alternatives
   - **Decision** — from the entry's Decision bullet
   - **Options Considered** — emit this section ONLY when the entry's Alternatives names a real rejected option (not `none`, not empty). Never infer it from prose elsewhere in the entry
   - **Consequences** — emit this section ONLY from the entry's own Consequences line, when present. Never infer it from Rationale or any other field
   - When the entry carries neither (Alternatives is empty/`none` and there is no Consequences line), the assembled ADR is short-form: field block + `### Context` + `### Decision` only — no `### Options Considered`, no `### Consequences`

2. Write ONE new fragment file `specs/_decision/NNN-<plan-name>.md`:
   - NNN = (count of existing files in `specs/_decision/`) + 1, zero-padded to 3 digits
   - H1: `# Decisions: <plan-name>`
   - Emit one `## ADR: <Title>` block per promoted entry, in decision-log order
   - MUST NOT edit any other file in `specs/_decision/` — a supersede reference is a one-way pointer from the new ADR's `**Supersedes:**` field to the target slug

3. Validate: `speq decision-log validate`

If `decision-log.md` is absent or has no "Promotes to ADR: yes" entries, skip silently.

## Record

Run `speq record <plan-name>` after the ADR fragment validates. The CLI merges every delta by its markers, strips the markers, validates each merged spec, archives the plan to `specs/_recorded/NNN-<plan-name>`, and rebuilds the search index. It rejects a plan with an invalid delta (an unrecognized anchor, a `CHANGED` or `REMOVED` block whose anchor is absent, two blocks on one anchor) before writing anything. On a non-zero exit, stop and report its message. Do not guess fixes, and do not merge or move files by hand.

## Finalize

1. Final validation: `speq feature validate`
2. If `specs/_recorded/NNN-<plan-name>/tasks.md` contains a `## PR Lifecycle` section, set `- [x] recorded` there. Section absent → skip silently (the plan did not run through the headless pipeline).

## Anti-Patterns

| Pattern | Why Wrong |
|---------|-----------|
| Merging deltas or moving the plan directory by hand | `speq record` is the tested implementation of the merge and archive; a hand merge skips its anchor checks |
| Assuming split/domain reorganization | User must decide |
| Filling an optional ADR section the entry does not carry | Rejected — `### Options Considered` and `### Consequences` are emitted only from the entry's own Alternatives/Consequences fields, never inferred from prose |
