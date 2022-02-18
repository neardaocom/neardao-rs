# Library crate
Shared crate Library provides necessary data types / functions for other NearDAO smart contracts and libs.

## Modules overview
### Workflow
- define workflow and its associated structures

### Types
- defines primitive types used in workflow and schema

### Expression
- kind of interpreter over primitive types
- enables to evaluate runtime expressions

### Data
- contains workflows, which are tested in unit_tests module and used for Workflow provider by (atm) just simply copying its json to workflow_provider load script. TBD: some kind of automatization for loading to the provider

### Utils
- provides functions to validate/bind/serialize almost any structure of json object defined by schema

### Storage
- defines storage structures

## TBD: 
1. FnMetadata schema
2. Storage deposit for self
3. Optimalizations all kinds in general
4. Crate module structure
5. Workflow validation tests