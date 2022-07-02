#!/bin/bash

##### USAGE #####
# A) Run without any args for building all libs
# B) Or with smart contract name you want to build
#################

set -e 

CRATES=(dao dao_factory workflow_provider staking fungible_token ft_factory)
FEATURES=""
ABS_BASEDIR=$(dirname $(readlink -f "$0"))

opt_dao_bin () {
    if [ -x "$(command -v wasm-opt)" -a $1 == "dao" ]; then
        echo "Optimize: dao"
        wasm-opt -Oz -o ${ABS_BASEDIR}/res/dao_opt.wasm ${ABS_BASEDIR}/res/dao.wasm
    fi
}

if [[ $1 == "dev" || $2 == "dev" ]]; then
    FEATURES='--features testnet';
fi

if [ ! -z "$1" -a -d "${ABS_BASEDIR}/contracts/${1}" ]; then

    LIB_DIR="${ABS_BASEDIR}/contracts/${1}"
    cd $LIB_DIR
    echo "Building: ${1}"  
    RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown $FEATURES --release

    if [ ! -e "${ABS_BASEDIR}/res" ]; then
        mkdir "${ABS_BASEDIR}/res"
    fi
    cp "${ABS_BASEDIR}/target/wasm32-unknown-unknown/release/${1}.wasm" "${ABS_BASEDIR}/res/"
    opt_dao_bin $1
else
    echo "BUILD SCRIPT: Building all contracts into res dir"

    for lib in "${CRATES[@]}"
    do
    :   
        echo "Building: $lib"      
        RUSTFLAGS='-C link-arg=-s' cargo build -p $lib --target wasm32-unknown-unknown --release
        cp "${ABS_BASEDIR}/target/wasm32-unknown-unknown/release/${lib}.wasm" "${ABS_BASEDIR}/res/"
        opt_dao_bin $lib
    done
fi

