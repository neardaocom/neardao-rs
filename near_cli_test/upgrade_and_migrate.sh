#!/bin/bash

# source ./near_cli_test/init_factory_and_dao.sh

 cp ~/.near-credentials/testnet/$CID.json ~/.near-credentials/testnet/$DCID.json; sed -i 's/dev-/dao.dev-/g' ~/.near-credentials/testnet/$DCID.json

# Build dao first then factory so it has new dao version
# Manual code changes must be done, otherwise same version will be used, to verify, just inc VERSION const
sh build.sh dao
sh build.sh

# Migrate factory to upload new dao version
near view $CID get_stats
source ./near_cli_test/migrate_factory.sh true
near view $CID get_stats

# Send NEARs to DAO to pay for storage
near send $CID $DCID 20

# Download & upgrade process for DAO with checks
near state $DCID
near call $DCID download_new_version '' --accountId $CID1 --gas 300000000000000 # ATM Costs 51 TGas
near view $DCID version_hash ''
near view $CID version_hash '{"version":0}' # 0 for latest version
near state $DCID
near view $DCID dao_config
near call $DCID upgrade_self '' --accountId $CID1 --gas 300000000000000 # ATM Costs 26 TGas
near state $DCID
near view $DCID dao_config
near view $DCID version_hash ''
near view $CID get_dao_list '{"from_index":0, "limit": 100}'
near view $DCID proposals '{"from_index":0, "limit": 100}'