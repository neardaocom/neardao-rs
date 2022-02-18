# Near DAO smartcontracts
- Metabuild 2 version

#### Crates:
1. Dao SC
2. DaoFactory SC
3. WorkflowProvider SC
4. Library
5. Tests - deprecated, will be replaced with Sandbox
## Compiling
- standardized build via contract-builder
- requires Docker  

## Testing

1. Unit tests (dao)
    - located in unit_tests for better readability
    - rust just by "cd" into dao crate and run by `cargo t` command

2. Via [NEAR CLI](https://docs.near.org/docs/tools/near-cli)
    - set of bash scripts located in near_cli_tests that utilises NEAR CLI tool
    - run by `. near_cli_test/<script-name>`
    - please, ALWAYS clean the testnets's resources used by running `. clean.sh`

## TBD
1. Finish Workflows
2. Unit tests
3. Refactoring and optimalizations all kinds
4. Testing env Kurtosis/Sandbox