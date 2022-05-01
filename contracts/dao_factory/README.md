# Dao factory contract

## Functions:
- deploy dao contract
- provide binaries for dao migration and upgrade

## Upgrade process (TBD)
- upgrade process requires dao has some free nears on its account to pay for temporary increased storage usage (2 free NEARs should be enough)

1. Download necessary binaries
    - Priviliged DAO member decides to upgrade dao so the member calls (via GUI) "download_migration" on DAO contract. Dao contract checks rights and makes promise to admin contract (defined in its settings - default is this factory), function "download_dao_migration" sending its current contract version. Factory checks provided version and makes promise to the caller, function "store_migration" with new migration code. Same process must be repeated with "download_dao_upgrade" on dao to store upgrade binary. Factory currently planned to store up to 5 latest versions.
    Dao stores migration and upgrade binaries under specific storage keys.
2. Migration
    - Privileged DAO member decides to start migration process. Call "deploy_migration" function on dao requires to have migration binary stored. Migrate function makes promise to itself to deploy migration version. Once deployed, then all migration must be done by calls to "migrate" function (or maybe another migrate_* functions defined in future) disabling all other functions on dao. It might be might be necessary to call "migrate" multiple times as DAO might have large amount of data and we are able to migrate only part of it because of the gas limit per function call. Once all data are migrated, then appropriate flag is set to let users know migration is done and upgrade can be started.
3. Upgrade
    - Again done by privileged DAO member. Call "upgrade" makes promise to itself, deploys new upgrade contract, cleaning migration code and unused structures and frees storage used by migration and upgrade binaries. Dao is usable once "upgrade" finishes.

## Relevant functions in factory contract:
1) download_dao_migration
2) download_dao_upgrade

## Relevant functions in dao contract:
1. store_migration
2. store_upgrade
3. download_migration
4. download_upgrade
5. deploy_migration
6. migrate
6. upgrade