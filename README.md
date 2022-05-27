<div align="center">

<img src="https://neardao.com/img/logo_neardao.png" alt="drawing" width="150"/>

<h1>neardao-rs</h1>

[![Build](https://github.com/neardaocom/app-rs/actions/workflows/build.yml/badge.svg?branch=devel%2Fnext)](https://github.com/neardaocom/app-rs/actions/workflows/build.yml)
[![Tests](https://github.com/neardaocom/app-rs/actions/workflows/tests.yml/badge.svg?branch=devel%2Fnext)](https://github.com/neardaocom/app-rs/actions/workflows/tests.yml)
    
</div>

## Smart contracts
- Dao
- Dao factory
- Workflow provider
- Staking
- Fungible token factory
- Fungible token
## Library
Contains shared definitions/types/functions used by NearDAO contracts. For more information, checkout [README](library/README.md)
## Building
- standardized build via contract-builder
- requires Docker

## Testing
1. Unit tests
    - located in unit_tests modules
    - run by `cargo test` all tests

2. Via [NEAR CLI](https://docs.near.org/docs/tools/near-cli)
    - set of bash scripts located in near_cli_tests that utilises NEAR CLI tool
    - run by `. tests/near_cli_test/<script-name>`
    - free the testnets's resources used by running `. clean.sh`

3. Integration tests
    - via[Near workpaces](https://github.com/near/workspaces-rs)

## Contributing
- TBD
