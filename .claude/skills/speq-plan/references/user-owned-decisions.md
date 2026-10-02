# User-Owned Decisions

`/speq-plan`, `planner-agent` (via `/speq-planning`), and `plan-reviewer` (via `/speq-plan-review`) share this definition. Each one says what it does with an unsettled decision.

## Definition

A user-owned decision is a choice where the user could reasonably pick a different option, and that does at least one of these:

- **Behavior:** changes what the feature does for a user: the input it accepts (partial, empty, invalid, or conflicting input included), its results, or which error appears, and when and where the caller sees it.
- **Compatibility:** changes behavior that existing callers rely on.
- **Interface:** adds to, removes from, or changes the public interface.
- **Design fork:** is irreversible, or picks between genuinely incompatible designs.
- **Sensitive data:** has a security or compliance consequence, such as where credentials or personal data are kept, for how long, or whether they are logged.
- **Scope and delivery:** sets what this plan covers and what it leaves to a follow-up, including nearby code with the same defect, or makes a release choice the target repository's rules leave to the author.

Every other choice belongs to the planner: naming, file placement, internal structure, test layout, a clear project convention.

## What Settles a Decision

A decision is settled only by a verbatim quote that states the choice, from one of these sources:

- the request
- the linked issue
- an answer in the brief's `## Clarifying Interview Results`
- a target-repository rule that leaves only one option
- the current code, when the choice is to keep behavior that already exists (cite `file:line`)

Three things never settle a decision:

- **A paraphrase or a source name alone.** "settled by: AGENTS.md" without a quote is open.
- **A rule that offers alternatives.** A rule that allows either a version bump or an `[Unreleased]` CHANGELOG entry leaves the choice open.
- **An unstated consequence.** When a specified approach changes something user-visible that the source does not mention, such as when an error appears or which status it carries, the approach is settled but the consequence is not. Ask about it as a consequence: "The issue's approach moves X to Y. Accept?"

## Recording

Record each settled decision in this form, one line each:

```
<decision>: <choice> (settled by: <source>: "<verbatim quote>")
```
