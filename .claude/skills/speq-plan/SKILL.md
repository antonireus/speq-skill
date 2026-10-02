---
name: speq-plan
description: Plan a feature or change through a clarifying interview, producing spec deltas, plan.md, and decision-log.md via planner-agent with adversarial review. Use when the user asks to plan, spec, design, or scope a new feature, a change or removal of existing behavior, a refactor, or a fix — before any implementation.
model: sonnet
effort: medium
---

# Spec Planner (Orchestrator)

This skill is a thin orchestrator. It runs the clarifying interview, collects context, and delegates the planning work (spec delta authoring, test mapping, task decomposition) to `planner-agent`.

## Required Skills (for the orchestrator)

Invoke before starting:
- `/speq-cli`: spec discovery and search

`planner-agent` and `plan-reviewer` invoke their own required skills.

## Workflow

### 0. Load Project Hook (orchestrator)

Check for `.speq/plan-hook.md` in the repo root.
- **Present:** read it. Announce "Loaded project hook: .speq/plan-hook.md". Its content is authoritative: it can add to, change, or override any part of this workflow. If the hook conflicts with this workflow, the hook wins.
- **Absent:** continue without mention.

### 1. Discovery (orchestrator)

Run the speq CLI:

```bash
speq domain list
speq feature list
speq search query "<relevant terms>"
```

If the request links a GitHub issue, fetch it with `gh issue view <number> --json title,body,comments`.

Read the code the change touches: each file, function, or type the request or linked issue names, and the direct callers of each. Use targeted reads (search for the symbol, then read its body), not whole files. Stop there. Discovery finds the decisions to ask about; `planner-agent` does the full exploration.

### 2. Clarifying Interview (orchestrator)

Read `<this skill's base directory>/references/user-owned-decisions.md` in full now. It defines the decisions the user owns and what settles one. The interview settles these decisions before planning starts, via `AskUserQuestion`. Never assume.

Run the interview in every permission mode, auto mode included: the interview is how this skill gets the user's decisions, not an interruption to avoid. A request that links an issue or proposes a fix still needs it, for the choices the issue leaves open and the scope edges. `/speq-plan-pr` is the path for planning without a live user.

1. **List the decisions.** Write down the decisions the plan has to make. Check every area in the definition against the code Discovery read, even when the request or linked issue specifies the approach. Add each user-visible consequence of the specified approach that the source does not state, for example an error that now appears earlier or carries a different status.
2. **Find a quote for each one.** A decision is settled only by a verbatim quote that states the choice, from a source the definition accepts. No quote, or a quote from a rule that offers alternatives, means the decision is open.
3. **Ask about every open decision.** Give MECE options, with your recommendation first. When the code shows why the choice matters, cite it as a counterexample (`file:line`). Ask about an unstated consequence as a consequence: "The issue's approach moves X to Y. Accept?" Do not ask the user to confirm the request itself, an approach the request or linked issue specifies, or an obligation the target repository's rules impose (a required CHANGELOG entry, for example).
4. **Record the rest.** Keep the user's answers verbatim for the brief. Write each settled decision to `## Orchestrator Assumptions` in the definition's recording form, with its quote, so `plan-reviewer` can check it. Details too small to ask about (naming, file placement, an obvious convention) go there too, marked `detail`.

### 3. Plan Name (orchestrator)

Pattern: `<verb>-<feature-scope>[-<qualifier>]`

| Verb | When |
|------|------|
| `add` | New feature |
| `change` | Modify existing |
| `remove` | Deprecate/delete |
| `refactor` | Restructure, same behavior |
| `fix` | Bug or spec mismatch |

If `specs/_plans/<plan-name>/` already exists, ask the user via `AskUserQuestion` whether to resume from it, replace it, or use a new name. Files left there by an earlier run were written without this interview's answers. Add the question and answer to the brief's `## Clarifying Interview Results`.

### 4. Delegate to planner-agent

Write the brief once, to `specs/_plans/<plan-name>/notes/brief.md` (create `notes/` if absent). Every sub-agent reads this one file, so no two of them work from different copies of the request. `notes/` is local scratch, kept out of every commit.

```
# Brief: <plan-name>

## Plan Name
<plan-name>

## User Intent
<the user's original request, verbatim, plus a 1-3 sentence summary>

## Clarifying Interview Results
<verbatim Q&A from the AskUserQuestion exchanges: only questions the user actually answered>

## Orchestrator Assumptions
<each decision you did not ask about, as `<decision>: <choice> (settled by: <source>: "<verbatim quote>")`, and small details you settled yourself, marked `detail`; one line each, or "none". These are open to challenge, unlike the interview answers>

## Templates
<this skill's base directory>/references/ (plan-template.md, delta-template.md, feature-template.md, decision-log-plan-template.md)

## Existing Context
<the exact `speq domain list` / `speq feature list` / `speq search query "..."` / `speq feature get` calls you ran, each followed by its output — name the query, not just the result. Then the code sites Discovery read, one `file:line` per line>

## External Research
<any research already conducted, or "none — agent to research as needed">

## Project Hook
<if active: note ".speq/plan-hook.md — read it and apply it" — otherwise omit this section>
```

Then spawn the planner with a short prompt:

```
Delegate to planner-agent — Plan <plan-name>

Brief: specs/_plans/<plan-name>/notes/brief.md

## Your Task
Produce spec deltas and plan.md per the `planner-agent` workflow. Tag tasks requiring deep reasoning with [expert] so the implementer orchestrator can route them to implementer-expert-agent.

Return the list of files created, the validation result, and a `Deviations from brief:` line.
```

After the spawn, say one line that the planner is running and end the turn. When a hand-back arrives for a result you already processed, say nothing.

### 5. Review planner-agent output (orchestrator)

When the sub-agent returns:

1. If the return starts with `OPEN QUESTIONS:`, ask each question via `AskUserQuestion`, with the planner's options and its recommended option first. Append each question and answer to the brief's `## Clarifying Interview Results`. Then respawn `planner-agent`: after a first pass, with the same short prompt and brief path; after a revision pass, with the revision message and `Scope: new interview answers`. Do this before the review in step 6, and again whenever a revision pass returns `OPEN QUESTIONS:`.
2. Read the sub-agent's `Validation:` line. Run `speq plan validate <plan-name>` yourself only when that line is missing or is not `pass`.
3. List all created files.

### 6. Adversarial Plan Review (orchestrator)

Spawn `plan-reviewer`, a diabolus advocatus, to challenge the plan before handoff. Maximum 2 rounds total.

**Round 1:**

```
Delegate to plan-reviewer — Review <plan-name> (round 1)

Brief: specs/_plans/<plan-name>/notes/brief.md
Planning note: specs/_plans/<plan-name>/notes/planning.md
Plan Artifacts: plan.md, decision-log.md, and every specs/_plans/<plan-name>/**/spec.md delta
Planner deviations: <the planner's `Deviations from brief:` line, or "none">
```

It writes its findings to `specs/_plans/<plan-name>/review/round-1.md` and returns only `PLAN REVIEW round 1: BLOCKERS: <n>, ADVISORY: <n>, INTENT: <n>, HUMAN: <n> — <path>`. `INTENT` counts the BLOCKERs on the Intent Fidelity axis alone; `HUMAN` counts every BLOCKER tagged `Escalation: HUMAN` per `/speq-plan-review`'s Escalation Class section — the rest are `MECHANICAL`.

**Before each respawn after a `plan-reviewer` verdict, tell the user what it found.** Print the counts, one line per BLOCKER from the round file (`[TAG]`, `Location`, the defect in a few words, `Escalation`), and the next step with its reason, for example "Respawning planner-agent to fix 2 MECHANICAL blockers". This is a status message, not a question: continue without waiting.

**If `INTENT > 0`:** the reviewer holds that the plan solves a different problem than the one asked. Read the Intent-Fidelity findings from the round file. Present them via `AskUserQuestion` before any revision. The user accepts the plan as-is, or gives guidance and you respawn `planner-agent` manually.

**If `INTENT == 0` and BLOCKER findings exist:**

1. Respawn `planner-agent` with the revision message below. `/speq-planning`'s Revision Mode defines what it does with the file.
2. Respawn `plan-reviewer` for round 2 with the brief and planning-note paths, the path to `review/round-1.md`, and a `Round 2 Scope:` field, so it confirms each round-1 BLOCKER is resolved before anything else. `MECHANICAL` means no human is needed to decide the fix, not that the fix needs no check: a fix can be partial or can contradict a `/speq-planning` rule.
   - `Round 2 Scope: confirm-only` when either of these holds:
     - round 1's `HUMAN` count was 0. This alone is enough; plan size does not matter.
     - the plan is small, meaning all three hold: the plan-name's verb, per the verb table, is `fix`; `plan.md` has no `## Design` section; `decision-log.md`'s `## Design Decisions` section is empty.
   - `Round 2 Scope: full` when neither holds. The reviewer then also checks for new findings.
3. Do not run a third adversarial round, even if round 2 raises new BLOCKERs.

**Revision message.** Every `planner-agent` respawn after a review uses exactly this shape and nothing more:

```
Revision Mode — <plan-name>
Findings file: specs/_plans/<plan-name>/review/round-<N>.md
Scope: <BLOCKERs | still-open MECHANICAL findings | new interview answers>
```

Do not restate or paraphrase findings, add instructions, or settle an open choice in the message. The file is the only source, and `/speq-planning`'s Revision Mode says what to do with it. A finding that leaves a choice open is the planner's to decide and report as a `Chose:` line, or to escalate. It is never yours to pre-decide.

**BLOCKERs remaining after round 2, split by `Escalation`:**

- **`MECHANICAL` remainder:** respawn `planner-agent` once more with the revision message pointing at `review/round-2.md`, scope `still-open MECHANICAL findings`. Don't interrupt the user for these unless one comes back `Could not resolve:` or `speq plan validate` fails — then it joins the `HUMAN` remainder below, with the `Could not resolve:` reason as the question.
- **`HUMAN` remainder** (plus any `MECHANICAL` finding a fix pass could not close): read those findings and use `AskUserQuestion`, one line of context each plus a pointer to the round file for detail. The user accepts the risk and proceeds, or gives guidance and you respawn `planner-agent` manually.

**ADVISORY findings** never loop and are never persisted. Read them from the last round file that ran a full pass and carry them into step 7's report: round 1's if round 2 ran confirm-only, round 2's otherwise. Drop any advisory the planner's return lists as `Fixed advisory:`.

### 7. Explain next steps (orchestrator)

Before you call the plan ready, check the last planner return and the last round file for a user-owned decision that is still open, such as an ADVISORY finding whose fix picks a behavior. Ask about each one via `AskUserQuestion`, append the answers to the brief, and respawn `planner-agent` with the revision message, the last round file, and `Scope: new interview answers`. Never list an open user-owned decision as an advisory in the report below.

Print the plan's summary per `/speq-plan-pr`'s `references/pr-body-template.md` — Current State / What Changes / Impact, composed from `plan.md`/`decision-log.md` the same way `speq-plan-pr` composes the PR body from them. This is terminal output, not a PR: print the `<details>` block's file list as a plain line, not the HTML fold, and the Test plan checklist as-is. `speq-plan-pr` and `speq-implement-pr` are the only skills that ever post this content as a PR; this step never touches git or GitHub.

Then:
- Report ADVISORY findings from step 6, read from the round file
- Report every `Deviations from brief:` entry from the planner's returns, and every `Chose:` line from a revision pass, one line each, so the user sees decisions the planner made without asking
- If step 6 fixed `MECHANICAL` findings, name it in one line ("N mechanical findings fixed and re-checked"). Do not restate each one
- Tell the user to run `/speq-implement <plan-name>` to continue
- Tell the user to run `/clear` to implement with a fresh context window
- If Claude Code is in "plan mode", call `ExitPlanMode` and ask to proceed with cleared context

## Spec Hierarchy (reference)

```
specs/
├── <domain>/<feature>/spec.md     # Permanent
├── _plans/<plan-name>/            # Active
└── _recorded/<plan-name>/         # Archived
```

## Work Split (reference)

| Step | Performed by | Why |
|------|--------------|-----|
| Discovery, interview, coordination | This skill (pins Sonnet) | Conversational, tool-call heavy |
| Spec delta authoring, ADR, task decomposition | `planner-agent` sub-agent | Reasoning-heavy; a defect here compounds through implementation |
| Adversarial review, revision loop | `plan-reviewer` sub-agent | Catches intent drift, infeasibility, and ambiguity before implementation |

Each sub-agent pins its own model and effort in its frontmatter, so planning quality does not depend on the parent session's configuration.

## Anti-Patterns

| Pattern | Why Wrong |
|---------|-----------|
| Authoring plan.md or spec deltas in the orchestrator | `planner-agent` owns all plan authoring |
| Skipping the clarifying interview, in any permission mode | Content comes from user answers, never assumptions; `/speq-plan-pr` is the non-interactive path |
| Moving an unsettled user-owned decision into `## Orchestrator Assumptions` | That section is for decisions a named source settles and details too small to ask about; a decision the user owns is an interview question |
| Interviewing from the issue text alone | The open choices sit in the code the change touches; read it in Discovery first |
| A third adversarial review round | Bounded to 2 — a `MECHANICAL` remainder gets one direct fix pass instead, a `HUMAN` remainder goes to the user |
| Asking the user about a `MECHANICAL` finding | Round count is not the escalation test — `Escalation: HUMAN` is; fix mechanical findings directly, no interruption |
| Skipping round 2 after a fix pass | An unchecked fix can ship a partial resolution; a `HUMAN: 0` round 1 gets a confirm-only round 2, not none |
| Persisting ADVISORY findings or looping on them | Report-only; they never gate |
| Pasting the interview, assumptions, or intent into a sub-agent prompt | The brief file is the single copy; a pasted copy drifts from it |
| Paraphrasing findings or choosing between options in a revision message | The findings file is the only source; a choice the orchestrator pre-decides is a design decision nobody reviewed |
| Replying to a repeated hand-back with "already handled" | Say nothing; the turn adds noise and no information |
| Embedding spec content in plan.md | Plans reference delta files |
