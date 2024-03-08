#!/bin/bash
# takes in one input for the commit msg and runs cargo run-script fix and then git commit -am "message from input" while logging what its doing
# usage: ./commit.sh "commit message"
# print usage and also check if there is a commit message if not error with message and exit
if [ -z "$1" ]
  then
    echo "No commit message supplied"
    echo "Usage: ./commit.sh \"commit message\""
    exit 1
fi
echo "Running cargo run-script fix"
cargo run-script fix
git add .
git commit -am "$1"
echo "Committed with message: $1"