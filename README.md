# Near DAO smartcontracts
- version 0.6

#### Smart contracts:
1. Dao
2. Dao Factory
3. Workflow Provider
## Compiling
- standardized build via contract-builder
- requires Docker

## Testing
1. Unit tests
    - located in unit_tests module
    - rust just by "cd" into dao crate and run by `cargo t` command

2. Via [NEAR CLI](https://docs.near.org/docs/tools/near-cli)
    - set of bash scripts located in near_cli_tests that utilises NEAR CLI tool
    - run by `. near_cli_test/<script-name>`
    - clean the testnets's resources used by running `. clean.sh`

## TBD
1. Finish Workflows
2. Unit tests
3. Refactoring and optimalizations all kinds
4. Testing env Kurtosis/Sandbox