<div align="center">

<img src="https://neardao.com/img/logo_neardao.png" alt="drawing" width="150"/>

<h1><code>neardao-rs</code></h1>
</div>
![build](https://github.com/neardaocom/app-rs/actions/workflows/build.yml/badge.svg?branch=devel/next) 
![checks](https://github.com/neardaocom/app-rs/actions/workflows/tests.yml/badge.svg?branch=devel/next)

## Smart contracts
- Dao
- Dao factory
- Workflow provider
- Staking (TBD)
- Media (TBD)
- FT (TBD)
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
    - run by `. near_cli_test/<script-name>`
    - free the testnets's resources used by running `. clean.sh`

3. [Near workpaces](https://github.com/near/workspaces-rs)
    - TBD

## Contributing
- TBD