#!/bin/bash

# 1. get the SEMVER from the Cargo.toml
SEMVER=$(sed -n 's/version = "\(.*\)"/\1/p' Cargo.toml)

# 2. remove all code from main.rs and replace with the specific text
echo "fn main() { println!(\"Hello, world!, from SEMVER: $SEMVER\"); }" > src/main.rs

# 3. write the SEMVER as a JSON object to semver.json
MAJOR=$(echo $SEMVER | cut -d. -f1)
MINOR=$(echo $SEMVER | cut -d. -f2)
PATCH=$(echo $SEMVER | cut -d. -f3)

echo "{ \"major\": $MAJOR, \"minor\": $MINOR, \"patch\": $PATCH }" > semver.json

# 4. Build the project and copy the artifacts
# Ensure the target directory is removed, then recreated
rm -rf ~/Downloads/itxtest
mkdir -p ~/Downloads/itxtest

cargo build --release

# Adjust this line to match the actual binary name produced by Cargo
# This often matches your Cargo project name unless specified otherwise
BINARY_NAME=$(basename $(pwd)) # This assumes the binary name matches the folder name

cp target/release/$BINARY_NAME ~/Downloads/itxtest
cp semver.json ~/Downloads/itxtest

# 5. Open the folder in Finder
open ~/Downloads/itxtest
