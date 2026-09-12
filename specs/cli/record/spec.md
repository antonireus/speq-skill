# Feature: CLI Record

The CLI SHALL provide a command to record approved plan deltas into permanent feature specs and archive the plan under a numbered folder.

## Background

* Command syntax: `speq record <plan-name>`
* Plans are located at `specs/_plans/<plan-name>/`
* Recorded plans are archived to `specs/_recorded/NNN-<plan-name>/`, where `NNN` is a zero-padded ordering number
* `NNN` is the count of existing entries under `specs/_recorded/` plus one, starting at `001`
* Duplicate `NNN` numbers across parallel plans are acceptable; `specs/_recorded/` is gitignored
* Delta markers: `<!-- DELTA:NEW -->`, `<!-- DELTA:CHANGED -->`, `<!-- DELTA:REMOVED -->`
* The anchor of a delta block is its first non-empty line
* Recognized anchors: `### Scenario: <name>`, `## Background`, and `# Feature: <name>`
* A `## Background` or `# Feature: <name>` anchor wraps that heading in place inside the delta file
* `DELTA:CHANGED` MAY target any recognized anchor
* `DELTA:NEW` and `DELTA:REMOVED` MAY target a `### Scenario: <name>` anchor only
* A `### Scenario: <name>` anchor matches the target spec heading of the same exact text
* A `# Feature: <name>` anchor matches the first `# Feature` heading of the target spec by prefix, so a delta MAY change the feature name in the heading
* The `<domain>/<feature>` directory path is unchanged by a heading rename
* A `## Background` or `# Feature: <name>` section ends at the next `## ` heading
* A recognized anchor MAY appear in at most one delta block per delta file, whatever the marker kinds
* Anchor checks are skipped for a delta spec whose target feature spec does not exist
* An anchor-check failure reports every violating block of the plan, one message per line
* No permanent spec is written until every delta spec of the plan has passed its anchor checks and merged
* Background and Feature-description changes are written to `notes/prose-realignment.md` in the plan directory before the plan is archived
* Exit code 0 on success, 1 on error

## Scenarios

### Scenario: Record new feature

* *GIVEN* a plan with a new feature delta at `specs/_plans/my-plan/cli/new-cmd/spec.md`
* *AND* no existing spec at `specs/cli/new-cmd/spec.md`
* *WHEN* the user runs `speq record my-plan`
* *THEN* the system SHALL copy the delta spec to `specs/cli/new-cmd/spec.md`
* *AND* the system SHALL strip all delta markers from the recorded spec

### Scenario: Merge NEW scenario

* *GIVEN* a delta with `<!-- DELTA:NEW -->` marker around a scenario
* *AND* an existing feature spec
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL append the new scenario to the existing spec's Scenarios section

### Scenario: Merge CHANGED scenario

* *GIVEN* a delta with `<!-- DELTA:CHANGED -->` marker around a scenario named "Login"
* *AND* an existing feature spec with a scenario named "Login"
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL replace the existing "Login" scenario with the delta version

### Scenario: Merge REMOVED scenario

* *GIVEN* a delta with `<!-- DELTA:REMOVED -->` marker around a scenario named "Guest login"
* *AND* an existing feature spec with a scenario named "Guest login"
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL remove the "Guest login" scenario from the spec

### Scenario: Archive plan after recording

* *GIVEN* a successful recording of plan `my-plan`
* *AND* `specs/_recorded/` already contains 2 archived entries
* *WHEN* the recording completes
* *THEN* the system SHALL move `specs/_plans/my-plan/` to `specs/_recorded/003-my-plan/`

### Scenario: Validate after merge

* *GIVEN* a plan with delta specs
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL validate each merged spec
* *AND* the system SHALL report any validation errors

### Scenario: Plan not found

* *GIVEN* no plan named `nonexistent` exists
* *WHEN* the user runs `speq record nonexistent`
* *THEN* the system SHALL report an error indicating the plan was not found
* *AND* the system SHALL exit with code 1

### Scenario: Recording fails on merge error

* *GIVEN* an existing feature spec
* *AND* a `<!-- DELTA:CHANGED -->` or `<!-- DELTA:REMOVED -->` block whose anchor is absent from that spec
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL report an error naming the missing anchor
* *AND* the system SHALL NOT archive the plan
* *AND* the system SHALL exit with code 1

### Scenario: Rebuild index after recording

* *GIVEN* a successful recording of plan `my-plan`
* *AND* a search index exists
* *WHEN* the recording completes
* *THEN* the system SHALL rebuild the search index
* *AND* the system SHALL display the number of scenarios indexed

### Scenario: Merge CHANGED Background section

* *GIVEN* an existing feature spec with a `## Background` section
* *AND* a delta with a `<!-- DELTA:CHANGED -->` block whose first line is `## Background`
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL replace the `## Background` section of the target spec with the block content
* *AND* the system SHALL keep the `## Scenarios` section that follows it
* *AND* the system SHALL keep one empty line between the replacement and the next heading

### Scenario: Merge CHANGED Feature description

* *GIVEN* an existing feature spec whose first heading is `# Feature: <name>`
* *AND* a delta with a `<!-- DELTA:CHANGED -->` block whose first line starts with `# Feature`
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL replace the heading and the description of the target spec with the block content
* *AND* the system SHALL keep the `## Background` section that follows it

### Scenario: Merge CHANGED Feature description that renames the heading

* *GIVEN* an existing feature spec whose first heading is `# Feature: <name>`
* *AND* a delta with a `<!-- DELTA:CHANGED -->` block whose first line is `# Feature: <other name>`
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL replace the heading of the target spec with `# Feature: <other name>`
* *AND* the system MUST NOT move or rename the `<domain>/<feature>` directory
* *AND* the system SHALL exit with code 0

### Scenario: Record a new feature spec without anchor checks

* *GIVEN* a plan whose delta has no target spec under `specs/`
* *AND* the delta wraps `## Background` in a `<!-- DELTA:NEW -->` block
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL write the target spec with the delta markers stripped
* *AND* the system MUST NOT report an anchor error
* *AND* the system SHALL exit with code 0

### Scenario: Reject DELTA:NEW targeting Background or Feature description

* *GIVEN* an existing feature spec
* *AND* a delta with a `<!-- DELTA:NEW -->` block whose first line is `## Background` or starts with `# Feature`
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL report an error naming the marker, the anchor, and `DELTA:CHANGED` as the correct marker
* *AND* the system SHALL NOT archive the plan
* *AND* the system SHALL exit with code 1

### Scenario: Reject DELTA:REMOVED targeting Background or Feature description

* *GIVEN* an existing feature spec
* *AND* a delta with a `<!-- DELTA:REMOVED -->` block whose first line is `## Background` or starts with `# Feature`
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL report an error naming the marker, the anchor, and `DELTA:CHANGED` as the correct marker
* *AND* the system SHALL NOT archive the plan
* *AND* the system SHALL exit with code 1

### Scenario: Reject delta block with unrecognized anchor

* *GIVEN* an existing feature spec
* *AND* a delta with a block whose first non-empty line is not a recognized anchor
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL report an error that lists the three recognized anchors and instructs the author to put the heading inside the marker
* *AND* the system SHALL NOT archive the plan
* *AND* the system SHALL exit with code 1

### Scenario: Reject two delta blocks with the same kind and anchor

* *GIVEN* an existing feature spec
* *AND* a delta file with two blocks of the same marker anchored at the same heading
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL report an error naming the repeated anchor
* *AND* the system SHALL NOT archive the plan
* *AND* the system SHALL exit with code 1

### Scenario: Reject two delta blocks of different kinds on the same anchor

* *GIVEN* an existing feature spec
* *AND* a delta file with a `<!-- DELTA:NEW -->` block and a `<!-- DELTA:CHANGED -->` block anchored at the same heading
* *WHEN* the user runs `speq record`
* *THEN* the system SHALL report an error naming the repeated anchor and both markers
* *AND* the system SHALL NOT archive the plan
* *AND* the system SHALL exit with code 1

### Scenario: Leave every target spec unchanged when one delta spec fails

* *GIVEN* a plan with two delta specs targeting two existing feature specs
* *AND* the first delta spec is valid and the second carries a block that the system rejects
* *WHEN* the user runs `speq record`
* *THEN* the system MUST NOT modify the target spec of the first delta spec
* *AND* the system SHALL NOT archive the plan
* *AND* the system SHALL exit with code 1

### Scenario: Write prose-realignment note when Background or description changes

* *GIVEN* a plan whose delta changes a `## Background` or `# Feature: <name>` section
* *WHEN* the recording completes
* *THEN* the system SHALL write `notes/prose-realignment.md` into the archived plan directory
* *AND* the note SHALL name the feature path and the anchor of each change
* *AND* the note SHALL contain the text before the change and the text after the change

### Scenario: Omit prose-realignment note when only scenarios change

* *GIVEN* a plan whose delta blocks all target `### Scenario: <name>` anchors
* *WHEN* the recording completes
* *THEN* the system MUST NOT write `notes/prose-realignment.md`
