# Feature: Inline Code

The driver SHALL accept connection strings like `exasol://host:port`.

## Background

Connection strings follow the format `exasol://host:port`.

* Default port: `8563`

## Scenarios

### Scenario: Parse `host:port`

* *GIVEN* a connection string `exasol://db:8563`
* *WHEN* the driver parses it
* *THEN* the driver SHALL read host `db` and port `8563`

### Scenario: Other scenario

* *GIVEN* setup
* *WHEN* action
* *THEN* result SHALL happen
