# Task Flow (Work Breakdown Structure)

Task management following **Work Breakdown Structure** decomposition. `tasks.md` is the single record of task state, and it survives context loss.

## tasks.md Format

```markdown
# Tasks: <plan-name>

## Phase 2: Implementation (Group A)
- [ ] 2.1 Implement login endpoint
- [~] 2.2 Implement logout endpoint

## Phase 2: Implementation (Group B)
- [ ] 2.3 Add auth middleware

## Phase 3: Verification
- [ ] 3.1 Run test suite
- [ ] 3.2 Run linter
```

**Status markers:**

| Marker | Status | Meaning |
|--------|--------|---------|
| `[ ]` | pending | Not started |
| `[~]` | started | Sub-agent working |
| `[x]` | completed | Verified done |

**PR Lifecycle section:** when the file starts with a `## PR Lifecycle` section, `speq-implement-pr` owns it as its checkpoint (see that skill's `references/checkpoint-protocol.md`). Its entries are unnumbered on purpose: task dispatch, Context Recovery, and implementer agents cover only the numbered `## Phase N` lines and never touch the lifecycle section.

## Task Lifecycle

```
tasks.md: [ ] → [~] → [x]
```

### Orchestrator Actions

| Event | tasks.md |
|-------|----------|
| Create tasks | Write file |
| Start group | `[ ]` → `[~]` |
| Task done | `[~]` → `[x]` |

## Parallelization

From plan's `## Parallelization` section. Each group is a knowledge cluster. One agent takes the whole group, routed by its hardest task (any `[expert]` task → `implementer-expert-agent`):

```markdown
## Parallelization

| Group | Tasks | Depends on | Knowledge |
|-------|-------|------------|-----------|
| A | 2.1, 2.2 | — | spec delta cli/auth; src/auth/, tests/auth_test |
| B | 2.3 | A | src/middleware/ |
| C | 2.4, 2.5 | A | spec delta cli/session; src/session/ |
```

**Execution order:**
1. Group A (one agent for the whole group)
2. Groups B and C (can run after A completes)

Older plans may lack the `Knowledge` column. Routing and execution order do not depend on it. Only the agent's orientation line does.

## Context Recovery

When resuming after context loss:

```
1. Read tasks.md
2. Find first non-[x] task in the ## Phase N sections (skip ## PR Lifecycle)
3. If [~], sub-agent was interrupted → restart that group
4. If [ ], group not started → begin normally
```

## Error Handling

| Situation | Action |
|-----------|--------|
| Sub-agent fails | Keep tasks as `[~]`, report error |
| Partial completion | Mark completed tasks `[x]`, retry others |
| Test failure | Do not mark `[x]`, fix and retry |

