---
name: speq-ext-research
description: External documentation and research via Context7 and WebSearch — library APIs and design patterns. Triggered by /speq-mission, planner-agent, implementer-agent, and implementer-expert-agent when current external sources are needed.
---

# External Research

## When to Use

| Source | Use For |
|--------|---------|
| Context7 | Library APIs, method signatures, usage examples |
| WebSearch | Design patterns, architecture decisions, best practices |

Context7 takes two calls: `resolve-library-id` (with `libraryName` and a `query`) returns the `libraryId` that `query-docs` needs. Prefer primary documentation over secondary commentary.
