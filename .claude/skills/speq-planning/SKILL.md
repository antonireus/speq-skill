---
name: speq-planning
description: Plan-authoring workflow — spec delta authoring, test mapping, plan.md/decision-log.md generation, expert-task tagging, user-owned decisions and their escalation, headless mode, and revision mode. Triggered by planner-agent.
---

# Plan Authoring

## Guiding Principles

- **BDD (Gherkin syntax)**: scenarios use GIVEN/WHEN/THEN; integration tests by default, unit tests only for isolated pure computation
- **EARS syntax**: spec narratives use unambiguous behavioral clauses
- **RFC 2119 keywords**: THEN steps use MUST, MUST NOT, SHALL, SHALL NOT, SHOULD, SHOULD NOT, MAY (uppercase)
- **ADR (Nygard format)**: design sections capture Goals / Non-Goals / Architecture / Trade-offs / Key Interfaces

## Workflow

### 1. Discover Existing Specs

The orchestrator already ran `speq domain list` / `speq feature list` / `speq search query` and passed the results as this brief's `## Existing Context` section. Treat that as the baseline. Do not re-run the same queries. Re-query only for gaps it does not answer:

```bash
speq feature get <domain>/<feature>   # a specific feature's full spec, if Existing Context only named it
speq search query "<narrower terms>"  # only if Existing Context's results do not cover a sub-area you need
```

**Search first** when modifying existing behavior. Check `Existing Context` before you assume a feature does not exist yet.

### 2. Author Spec Deltas

For each feature in scope:

```
specs/<domain>/<feature>/spec.md exists?
├─ Yes → DELTA markers (/speq-plan's references/delta-template.md)
└─ No  → Full spec (/speq-plan's references/feature-template.md)

Output: specs/_plans/<plan-name>/<domain>/<feature>/spec.md
```

**Prose drift check**: after drafting a scenario delta for an existing feature, re-read that feature's `## Background`, its `# Feature: <name>` description, and its other recorded scenarios (`speq feature get <domain>/<feature>`). When the scenario change makes any of them inaccurate, author a `DELTA:CHANGED` block for it too, per `/speq-plan`'s `references/delta-template.md`.

### 3. Test Mapping and Verification

Every scenario requires two forms of external proof. No claims, only evidence.

**Tests** (one per scenario):
- Map each scenario to a test (file path + test name). Use an integration test by default
- Use a unit test instead only when the scenario's behavior is pure computation with no I/O; mark it `Unit` in Scenario Coverage
- One test per scenario by default; combine only when scenarios share setup and assertions
- Confirm a Checklist command actually compiles and runs each mapped test: check feature flags, conditional compilation, skip/ignore markers, and test targets

**Manual invocation** (mandatory per feature):
- Concrete commands that invoke the built software
- Expected observable output per command

### 4. Generate plan.md

Populate plan.md per `/speq-plan`'s `references/plan-template.md`:

1. **Context**: why the change is being made
2. **Features**: table referencing spec delta files (NEVER embed spec content), immediately followed by an Impact entry describing user/operator-facing consequences (breaking changes called out, or "None")
3. **Design**: ADR for new features / major changes; skip for minor fixes. For a new abstraction or module boundary, justify it against `/speq-design-philosophy`'s Quick Diagnostic (deep vs. shallow, dependency direction)
4. **Tasks**: work breakdown in implementation order
5. **Parallelization**: knowledge clusters, per the rules below
6. **Verification**: Scenario Coverage + Manual Testing + Checklist (from `specs/mission.md`)

**Parallelization groups are knowledge clusters.** Group tasks vertically: one spec delta plus the code area it governs. Never group by layer. Layer slices (fixtures, then module, then CLI, then tests) make each layer's agent re-derive the same mental model; a vertical slice orients one agent once. Each group row declares a `Knowledge` column: the group's spec delta path(s) plus the source and test files they govern. The implement orchestrator hands this entry to the group's agent as its orientation pointer. Two facts govern the grouping:

1. Tasks that share a spec delta or a source module default into one group.
2. Overlapping `Knowledge` entries across groups are a consolidation signal, not a parallelism opportunity.

If two clusters would contend on one shared file, split the module per feature (often the better design; check it against `/speq-design-philosophy`) or declare a dependency and sequence the clusters. Parallelism is a side effect, not the goal.

### 5. Generate decision-log.md

Create `specs/_plans/<plan-name>/decision-log.md` from `/speq-plan`'s `references/decision-log-plan-template.md`.

**What to capture:**
- **Interview section**: verbatim or close paraphrase of every Q&A exchange in the brief's `## Clarifying Interview Results`. Record `## Orchestrator Assumptions` entries as Design Decisions entries instead; they are not user answers
- **Design Decisions section**: one entry per significant choice made while authoring spec deltas or plan.md (architecture patterns, rejected alternatives, scope boundaries)
- **Review Findings section**: leave empty; populated in Revision Mode after `plan-reviewer` blockers, and by `speq-implement` after code review

**Promotion gate.** `Promotes to ADR: yes` requires one of:

- an **architecture or design constraint** that binds future work beyond this plan and that no scenario can express (which module owns a concern, a boundary, a data format);
- a **reversal of a decision recorded in `specs/_decision/`**. Name the superseded decision.

Behavior that the plan's scenarios specify stays `no`: the spec and the CHANGELOG already record it. This includes a fix that brings behavior in line with what users would expect, such as removing an unsafe default. An alternative named in an issue or in an `Alternatives:` line does not make a decision ADR material. Procedural and workflow decisions default to `no`. The only override: a project-wide process convention that (a) binds every future plan, (b) is not scoped to just this plan, and (c) is not a corollary of another decision — the entry's Rationale MUST state the override explicitly. A corollary of an already-promoted decision is not its own entry: record it as a bullet in that parent entry's `Consequences` line instead.

Set `Promotes to ADR: no` for local design choices, scope trims, and implementation details. This applies to every decision entry, Design Decisions and Review Findings alike: one ADR per genuinely new project-wide constraint, never one per entry or per resolved finding. A plan with ten Design Decisions entries or ten resolved blockers does not owe ten ADRs — most decisions in a normal plan are local (this file's structure, this feature's naming) and stay `no`. Never promote a local file-placement or process detail (where a note lives, how a plan's own scratch state is organized) just because it was deliberate enough to write down.

**Review Findings entries lean further to `no`.** A `[plan-review]`-prefixed entry (Revision Mode, below) records a mistake this plan made and then corrected, not automatically a project-wide convention. Apply the same gate: promote only when the fix itself passes the gate above, or meets the process-convention override — not merely because the finding was hard to resolve. Never promote a process or tooling workaround specific to this plan's own mechanics (a spec-merge gap, a review-loop correction, a task-reordering fix), or a one-off bug fix with no bearing outside this plan.

If a decision supersedes an earlier one, name the superseded decision's title in the entry. `recorder-agent` maps that title to the superseded ADR's slug at promotion.

### 6. Expert-Task Tagging

Tag tasks that require deep reasoning with `[expert]` at the end of the task line:

```markdown
- [ ] 2.1 Add CLI flag parsing
- [ ] 2.2 Implement lock-free queue for concurrent spec writes [expert]
- [ ] 2.3 Write integration tests for flag combinations
- [ ] 2.4 Refactor validator to preserve ordering invariants across plugins [expert]
```

**Use `[expert]` only when the task genuinely needs it:**
- Concurrency / ordering / race conditions
- Cross-file refactors with behavioral dependencies
- Novel algorithms or non-obvious correctness
- Security-sensitive code paths

**Do NOT tag as expert:**
- Standard CRUD, CLI flag plumbing, test fixtures
- Copy-paste from existing patterns
- Documentation or config changes

If any task in a parallelization group carries the tag, the orchestrator routes the whole group to `implementer-expert-agent`; all-untagged groups go to `implementer-agent`. One tag prices its entire group at the expert model, while a missing tag on genuinely hard work risks defects. Most tasks stay untagged.

### 7. Validate Plan

Before returning, and only once `plan.md` and `decision-log.md` exist (the validator reports an error while `plan.md` is missing):

```bash
speq plan validate <plan-name>
```

Write each scenario with at most 3 AND steps from the start, so the validator has nothing to split.

Fix any failures and warnings. Fix a too-many-AND-steps warning by splitting the scenario. Never merge steps into one compound step or drop a requirement step to get under the limit. Common fixes:
- Close unclosed delta markers with `<!-- /CHANGED -->`, `<!-- /NEW -->`, `<!-- /REMOVED -->`
- Uppercase RFC 2119 keywords
- Fix step formatting (bold keywords: `*GIVEN*`, `*WHEN*`, `*THEN*`, `*AND*`)

### 8. Write Planning Hand-off Note

Before you return to the orchestrator, write `specs/_plans/<plan-name>/notes/planning.md` (create the `notes/` directory if absent). Write a plain list, not a narrative:

- Files and symbols you checked
- Searches you ran (`speq search`, Serena queries), each with its target

Do not write: why, invariants as prose, conventions commentary, or decisions. Those already live in `decision-log.md`. This note is a map of what you checked, not a story about it. A map stays true even after a later revision changes a decision; a story does not.

A revision-mode respawn (below) reads this note first, before it re-reads `plan.md`, `decision-log.md`, or spec deltas, and before it re-explores the codebase.

This note stays out of every commit. It is local scratch, never evidence.

### 9. Pre-Return Self-Check

Before you return to the orchestrator, run this checklist once against your own `plan.md`, `decision-log.md`, and spec deltas. Answer each line against the artifacts on disk, not from memory. Fix anything that does not hold before you return.

This is prevention, not the review gate. `plan-reviewer` still runs next, full-strength, unchanged.

- Every answer in `## Clarifying Interview Results` shows up in a scenario, a task, or a Design Decision, and nothing in the plan lacks a traceable user need.
- Every user-owned decision the plan makes is settled by the brief. In headless mode, each one you decided yourself has a Design Decisions entry whose Rationale says so.
- Every mapped test is compiled and run by a Checklist command (step 3).
- For each existing feature you changed, its Background, description, and other recorded scenarios still hold, or carry a `DELTA:CHANGED` block (step 2's Prose drift check).
- Every Background and description line you wrote states a fact some scenario step depends on, not how the code implements it.
- Every item in `plan.md`'s Impact has a scenario, and, when the target repository keeps a CHANGELOG, an entry in the task that writes it.
- No two scenarios in this plan contradict each other.
- Each scenario covers one case: no either/or inputs or outcomes, and no step that merges several conditions or assertions.
- Every spec delta has an implementing task, and every task traces to a delta or a rule in the target repository's `CLAUDE.md`/`AGENTS.md` (a CHANGELOG entry, for example).
- Where the change touches security, performance, migration, or concurrency, a scenario or task covers it.
- Every `Promotes to ADR: yes` entry passes the promotion gate (step 5).

`/speq-plan-review` holds the full finding taxonomy `plan-reviewer` applies next.

## User-Owned Decisions

Read `/speq-plan`'s `references/user-owned-decisions.md` before you author anything. It defines which decisions the user owns and what settles one. A decision the brief settles carries a verbatim quote from its source. An `## Orchestrator Assumptions` entry without such a quote settles nothing. A user-visible consequence of a specified approach that the source does not state is not settled either, even when the approach is.

Every other choice is yours. Make it, and record the significant ones under Design Decisions.

What happens to a user-owned decision the brief does not settle depends on the mode:

- **Interactive** (the orchestrator's prompt has no `Interview Mode` field): the user decides it, not you. Finish the code exploration first, so you return every such decision at once. Then write `notes/planning.md` (step 8) and return the escalation below before you author any artifact that depends on these decisions. The orchestrator asks the user, appends the answers to the brief, and respawns you with the same brief.
- **Headless** (`Interview Mode: headless`): follow Headless Mode below.

**Escalation format (both modes).** Return the response prefixed with the exact sentinel `OPEN QUESTIONS:` followed by a markdown bullet list, one bullet per decision: the question, why the brief does not settle it (with a file and line when the code shows it), two to four options, and the option you recommend. Do not mix this sentinel into a normal completion report. The one exception is Revision Mode, where the `Resolved:` and `Could not resolve:` lines follow the question list.

## Headless / Non-Interactive Mode

If the orchestrator's prompt states `Interview Mode: headless` (used by `speq-plan-pr`, never by the interactive `speq-plan`), there is no human to ask mid-planning. This section replaces the interactive rule for unsettled user-owned decisions:

- **Assume and document.** For conventions, naming, implementation details, and any choice with a clearly conventional default: make the call and record it as a `decision-log.md` entry (Rationale explains why this default). This is the common case; most headless plans finish without escalating.
- **Escalate only irreducible decisions**: irreversible ones, changes to what the feature does for a user, genuinely incompatible architectural designs, or security/compliance. Before escalating, save every file completed so far (plan.md, delta specs, decision-log.md) exactly as it stands. The orchestrator persists this partial state for human review, so it must be usable as-is.
- **Escalation format.** Use the escalation format in User-Owned Decisions. Headless mode changes when to escalate, not the quality of what you escalate.

## Revision Mode

If the orchestrator respawns `planner-agent` with the path to a `plan-reviewer` findings file instead of a fresh planning brief:

- Read `specs/_plans/<plan-name>/notes/planning.md` first, if present. This is your own prior-pass hand-off note. It orients you before you re-read anything else.
- Read the BLOCKER list from the path given in the prompt: `specs/_plans/<plan-name>/review/round-<N>.md`. The findings never arrive inline; the file is the only source.
- When the prompt's scope is `new interview answers`, apply only the answers the brief's `## Clarifying Interview Results` gained since your last pass. Change only what those answers decide, record each in `decision-log.md`'s Interview section, and re-run `speq plan validate <plan-name>`. The findings file is context, not a work list, in this scope.
- Resolve every BLOCKER finding. Each `Fix:` line names the artifact, section, and concrete change; treat it as the reviewer's proposal, not a waiver of this skill's rules. Apply it together with the rest of this workflow, the Prose drift check included. When a `Fix:` line contradicts a rule here or in `/speq-plan`'s `references/delta-template.md`, follow the rule and say so in the `Resolved:` evidence. Leave content no finding touches as it is.
- You MAY also fix an ADVISORY finding when it corrects a fact you can verify in the repository (a count, a path, a test name, a missing task a project rule requires) and the change stays local. Report each as `Fixed advisory: <title>: <evidence>`.
- For each blocker resolved, add a `## Review Findings` entry to `decision-log.md` titled `[plan-review] <short finding title>`, with **Finding** (what `plan-reviewer` flagged), **Direction change** (what changed), and **Promotes to ADR** (per the rule above).
- Re-run `speq plan validate <plan-name>` before returning.
- When a finding's `Fix:` leaves a choice open (a default, a boundary, which of two behaviors), decide it by the brief and the interview, record it as a `decision-log.md` entry, and return `Chose: <title>: <decision> (decision-log [<n>])`. The orchestrator shows these to the user. If the choice is a user-owned decision the brief does not settle, handle it per User-Owned Decisions instead: in interactive mode, escalate it and do not pick.
- Return one line per blocker you addressed: `Resolved: <title>: <evidence>` or `Could not resolve: <title>: <why>`. The orchestrator uses these lines to decide the next step, so include them even when every finding resolved cleanly.
- Before you return, collect every user-owned decision the plan still leaves open, wherever it came from: a BLOCKER's fix, an ADVISORY finding you did not apply, or your own pass. In interactive mode, return all of them as an `OPEN QUESTIONS:` block. Never leave one for the orchestrator to report as an advisory. In headless mode, follow Headless Mode.
