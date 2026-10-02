# Feature: Spec Hierarchy

The system SHALL organize feature specifications in a shallow two-level hierarchy of domain and feature to improve discoverability and scalability.

## Background

* Specs are stored at `specs/<domain>/<feature>/spec.md`
* Domains group related features (e.g., `cli/`, `validation/`)
* Maximum nesting depth is 2 levels (domain/feature)
* The `_plans/` and `_recorded/` directories are reserved for plan management
* Domain and feature names use kebab-case
* The project root is the nearest directory, starting at the working directory and walking up, that contains `specs/`
* Every `speq` command resolves `specs/` against the project root

## Scenarios

### Scenario: Valid spec path structure

* *GIVEN* a feature specification file
* *WHEN* the system resolves the spec path
* *THEN* the path SHALL follow the pattern `specs/<domain>/<feature>/spec.md`

### Scenario: Discover all domains

* *GIVEN* a specs directory with multiple domain subdirectories
* *WHEN* the system lists domains
* *THEN* the system SHALL return all immediate subdirectories except `_plans` and `_recorded`

### Scenario: Discover features in domain

* *GIVEN* a domain directory containing feature subdirectories
* *WHEN* the system lists features in that domain
* *THEN* the system SHALL return all subdirectories containing a `spec.md` file

### Scenario: Discover all features

* *GIVEN* a specs directory with domains and features
* *WHEN* the system discovers all features
* *THEN* the system SHALL return all features across all domains
* *AND* each feature SHALL include its domain and feature name

### Scenario: Reserved directories excluded

* *GIVEN* a specs directory containing `_plans/` and `_recorded/` directories
* *WHEN* the system discovers domains or features
* *THEN* the system SHALL NOT include `_plans` or `_recorded` as domains

### Scenario: Empty domain ignored

* *GIVEN* a domain directory with no feature subdirectories
* *WHEN* the system discovers features
* *THEN* the system SHALL NOT include the empty domain in results

### Scenario: Run from a subdirectory of the project

* *GIVEN* a project whose root contains `specs/`
* *WHEN* the user runs a `speq` command from a subdirectory of that project
* *THEN* the system SHALL resolve `specs/` against the project root
* *AND* the command SHALL behave as if run from the project root

### Scenario: No specs directory in any parent

* *GIVEN* a working directory with no `specs/` directory in it or any parent directory
* *WHEN* the user runs a `speq` command
* *THEN* the system SHALL report that no `specs/` directory was found and name the working directory
* *AND* the system SHALL exit with code 1
