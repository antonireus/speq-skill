# Feature: AND Step Limit

Checks the AND-step warning.

## Background

The system is ready.

## Scenarios

### Scenario: Four assertions after THEN

* *GIVEN* the system is ready
* *WHEN* an action occurs
* *THEN* the system SHALL respond
* *AND* the system SHALL log the action
* *AND* the system SHALL update the counter
* *AND* the system SHALL notify the owner
* *AND* the system SHALL close the request
