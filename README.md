# Near DAO prototype

- version 0.4

## Testing

Run only simulation tests:  
`sh run_tests.sh`  

Run all tests:  
`sh run_tests.sh all`

### Test types

1. Unit tests (dao)
    - located in unit_tests for better readability
    - rust just by "cd" into dao crate and run by `cargo t` command

2. Simulation tests
    - **requires to add "rlib" to Cargo.toml in "dao" and "dao-factory" crates**
    - suitable for measuring gas fees and storage usage
    - testing "time-dependent features" requires good amount of tweaking dao configs, it is better to use near cli for this purpose
    - TODO automate with in run_tests.sh

3. Via [NEAR CLI](https://docs.near.org/docs/tools/near-cli)
    - set of bash scripts located in near_cli_tests that utilises NEAR CLI tool
    - run by `. near_cli_test/<script-name>`
    - please, ALWAYS clean the testnets's resources used by running `. clean.sh`