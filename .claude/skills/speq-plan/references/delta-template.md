# Delta Specification Template

Captures feature changes in plans before applying to permanent specs.

## Rules

1. **New Feature**: Full spec (Feature heading, Background, all Scenarios)
2. **Existing Feature**: Wrap only what changed in markers. A `### Scenario:` heading, `## Background`, or the `# Feature: <name>` description — each wrapped in place, heading included inside the marker
3. **Applying**: Merge deltas into permanent specs, remove markers

## Anchors

The anchor of a delta block is its first non-empty line. `record` and `plan validate` read only that line to decide what the block targets — nothing else in the block affects it.

| Anchor | First line of the block | Match |
|--------|--------------------------|-------|
| Scenario | `### Scenario: <name>` | Exact |
| Background | `## Background` | Exact |
| Description | `# Feature: <name>` | Prefix (`# Feature`) — a description delta MAY rename the feature |

Put the heading itself inside the marker, not above it. A block whose first line is not one of these three is an unrecognized anchor and `record`/`plan validate` reject it.

## Markers

```
<!-- DELTA:NEW -->     <!-- /DELTA:NEW -->
<!-- DELTA:CHANGED --> <!-- /DELTA:CHANGED -->
<!-- DELTA:REMOVED --> <!-- /DELTA:REMOVED -->
```

`DELTA:NEW` applies to scenarios only — Background and the description are required sections that always exist, so there is nothing to add; use `DELTA:CHANGED` to change them. `DELTA:REMOVED` applies to scenarios only, for the same reason: removing a required section produces an invalid spec.

## Deletion Semantics

A `DELTA:CHANGED` block on `## Background` or `# Feature: <name>` replaces the whole section. Any line the block omits is deleted from the permanent spec. Copy the current section (`speq feature get --raw <domain>/<feature>`), then edit it — never write only the lines that changed.

## Example: Scenarios

```markdown
## Scenarios

<!-- DELTA:NEW -->
### Scenario: User resets password

* *GIVEN* a registered user exists
* *WHEN* the user requests a password reset
* *THEN* the system SHALL send a reset link
<!-- /DELTA:NEW -->

<!-- DELTA:CHANGED -->
### Scenario: User logs in

* *GIVEN* a registered user exists
* *WHEN* the user submits valid credentials
* *THEN* the system SHALL enforce two-factor authentication
<!-- /DELTA:CHANGED -->

<!-- DELTA:REMOVED -->
### Scenario: Guest checkout
<!-- /DELTA:REMOVED -->
```

## Example: Background

Use this when a scenario change makes the current Background prose inaccurate. Copy the section's current content in full, then edit it — per Deletion Semantics above, anything left out is gone from the permanent spec.

A delta file whose only blocks are `## Background` or `# Feature` MUST also include one scenario copied verbatim from the target spec, outside any delta marker: `speq plan validate` rejects a delta file with no scenarios, and `speq record` ignores unmarked content, so the unmarked scenario satisfies validation without itself being merged as a change.

```markdown
# Feature: CLI Record

The CLI SHALL provide a command to record approved plan deltas into permanent specs.

<!-- DELTA:CHANGED -->
## Background

* Command syntax: `speq record <plan-name>`
* Merges every delta spec of the plan before writing any permanent spec
<!-- /DELTA:CHANGED -->

## Scenarios
```

## Example: Description

Wrap the `# Feature: <name>` heading itself inside the marker. Because the match is a prefix on `# Feature`, the delta MAY change the feature name at the same time.

```markdown
<!-- DELTA:CHANGED -->
# Feature: CLI Record

The CLI SHALL merge every approved delta of a plan into permanent specs in one atomic write, then archive the plan.
<!-- /DELTA:CHANGED -->

## Background
```

## Anti-Pattern: Heading Outside the Marker

```markdown
## Background
<!-- DELTA:CHANGED -->
* Command syntax: `speq record <plan-name>`
<!-- /DELTA:CHANGED -->
```

Wrong. The block's first non-empty line is the bullet, not `## Background`, so the anchor is unrecognized and `record`/`plan validate` reject it. Move the heading inside the marker, as in the Background example above.
