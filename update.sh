#!/bin/bash

# 0. use cargo bump to bump patch version
cargo bump patch

# 1. get the SEMVER from the Cargo.toml
SEMVER=$(cargo get package.version)

BINNAME=$(cargo get package.name)

# 3. write the SEMVER as a JSON object to semver.json
MAJOR=$(echo $SEMVER | cut -d. -f1)
MINOR=$(echo $SEMVER | cut -d. -f2)
PATCH=$(echo $SEMVER | cut -d. -f3)

echo "{ \"major\": $MAJOR, \"minor\": $MINOR, \"patch\": $PATCH }" > semver.json

cargo build --release

# Adjust this line to match the actual binary name produced by Cargo
# This often matches your Cargo project name unless specified otherwise

# cp target/release/$BINARY_NAME ~/Downloads/itxtest
# cp semver.json ~/Downloads/itxtest

# 5. Open the folder in Finder
# open ~/Downloads/itxtest

# 6. upload to az blob storage
az storage blob upload --account-name blobperma --container-name blob-bin --name bintest --file target/release/$BINNAME --overwrite --auth-mode login &
az storage blob upload --account-name blobperma --container-name blob-bin --name semver.json --file semver.json --overwrite --auth-mode login 