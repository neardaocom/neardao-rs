# Dao factory contract

## Functions:
1. deploy dao contract  
2. provide binaries for dao migration and upgrade to new version

## Upgrade process
Upgrade process requires dao has some free nears on its account to pay for temporary increased storage usage (2 free NEARs should be enough).

1. Download necessary binaries
    - Priviliged DAO member decides to upgrade dao so the member calls (via GUI) "download_migration_and_upgrade" on DAO contract. Dao contract checks rights and makes promise to admin contract (defined in its settings - default is this factory), function "download_migration_and_upgrade" sends its current contract version. Factory checks provided version and makes promise to the caller, function "store_migration_bin" with new migration code and "store_upgrade_bin" with new version code.  Factory currently stores up to 5 latest versions.
    Dao stores migration and upgrade binaries under specific storage keys.
2. Migration
    - Privileged DAO member decides to start migration process. Call "start_migration" function on dao requires to have both migration and new version (upgrade) binaries stored. Migrate function makes promise to itself to deploy migration version. Once deployed, then all migration must be done by calls to "migrate_data" function disabling all other functions on dao. It might be necessary to call "migrate_data" multiple times as DAO might have large amount of data and we are able to migrate only part of it because of the gas limit per function call. Once all data are migrated, then appropriate flag ("TODO") is set to let users know migration is done and upgrade can be started.
3. Upgrade
    - Triggered by privileged DAO member. Call "upgrade" makes promise to itself, deploys new upgrade contract, cleans upgrade code from the storage and unused structures. Dao is usable once "upgrade" finishes.

## Relevant factory contract functions:
1. download_new_version

## Relevant dao contract functions:
1. download_migration_and_upgrade
2. store_migration_bin
3. store_upgrade_bin
4. start_migration
6. deploy_migration_bin
7. migrate_data
8. upgrade
8. deploy_upgrade_bin