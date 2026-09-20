# Decision Log: <plan-name>

<!--
STRUCTURAL TEMPLATE - DO NOT COPY-PASTE
Generate actual content from the clarifying interview and plan design.
Capture interview Q&A verbatim or close paraphrase.
Promotion gate: mark "Promotes to ADR: yes" only for a change in behavior, architecture, or design.
Procedural and workflow decisions default to "no". The only override is a project-wide process
convention that (a) binds every future plan, (b) is not scoped to just this plan, and (c) is not a
corollary of another decision — state the override explicitly in Rationale when you invoke it.
A corollary of an already-promoted decision is not its own entry: add it as a bullet in that
parent entry's Consequences line instead.
-->

## Interview

<!-- Q&A from the clarifying interview. One Q/A pair per exchange. -->

**Q:** <question asked>
**A:** <user answer>

## Design Decisions

<!-- Key design choices made in this plan. One entry per significant decision. -->

### [1] <Short decision title>

- **Decision:** What was chosen.
- **Alternatives:** What else was considered and why rejected. May read `none`.
- **Rationale:** Why this choice.
- **Consequences:** Effects, trade-offs, or corollary decisions folded in here. Omit this line entirely when there are none. <!-- optional -->
- **Promotes to ADR:** yes

### [2] <Short decision title>

- **Decision:** What was chosen.
- **Alternatives:** What else was considered and why rejected. May read `none`.
- **Rationale:** Why this choice.
- **Consequences:** Effects, trade-offs, or corollary decisions folded in here. Omit this line entirely when there are none. <!-- optional -->
- **Promotes to ADR:** no

## Review Findings

<!-- Significant review findings that changed direction: plan-review findings (prefix title "[plan-review]"), populated by speq-plan/speq-plan-pr after plan-reviewer resolves a blocker, and code-review findings, populated by speq-implement after code review. -->

### [1] <Finding title>

- **Finding:** What the reviewer identified.
- **Direction change:** How the implementation was adjusted.
- **Promotes to ADR:** yes / no
