# Contract Builder
Modified version from: https://github.com/near/near-sdk-rs/tree/master/contract-builder

This is a helper Dockerfile that allows to build contracts in a reproducible way.

The contract built in the Docker will result in a binary that is the same if built on other machines.

For this you need to setup Docker first.

## Build container

```bash
./build.sh
```

## Start docker instance

By default, the following command will launch a docker instance and will mount this repository root under `/host`.

```bash
./run.sh
```

If you need to compile some other contracts, you can first export the path to the contracts, e.g.

```bash
export HOST_DIR=/root/contracts/
```

## Build contracts in docker

Enter mounted path first:

```bash
cd /host
```
Usage is same as with build.sh.
To build all contracts do the following

```bash
./build.sh 
```
To build only dao contract
```bash
./build.sh dao
```
To build only dao_factory contract:

```bash
./build.sh dao_factory
```
