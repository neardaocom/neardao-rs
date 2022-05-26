#!/bin/bash

set -e 

CRATES=(upgrade_dao_v1 upgrade_dao_v2_migration upgrade_dao_v2 upgrade_dao_factory_v1 upgrade_dao_factory_v2_migration upgrade_dao_factory_v2)
ABS_BASEDIR=$(dirname $(readlink -f "$0"))
TARGET_DIR="${ABS_BASEDIR}/../res_upgrade/"

echo "BUILD SCRIPT: Building upgrade mocks into res_upgrade dir"

for lib in "${CRATES[@]}"
do
:   
    echo "Building: $lib"      
    RUSTFLAGS='-C link-arg=-s' cargo build -p $lib --target wasm32-unknown-unknown --release
    cp "${ABS_BASEDIR}/../../target/wasm32-unknown-unknown/release/${lib}.wasm" $TARGET_DIR
done
