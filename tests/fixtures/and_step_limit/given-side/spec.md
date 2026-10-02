# Feature: AND Step Limit

Checks the AND-step warning.

## Background

The system is ready.

## Scenarios

### Scenario: Four preconditions before WHEN

* *GIVEN* the system is ready
* *AND* a user exists
* *AND* the user is signed in
* *AND* the user owns a project
* *AND* the project has one open request
* *WHEN* the user closes the request
* *THEN* the system SHALL mark the request closed
