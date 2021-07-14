#!/bin/bash

set -e
set -x

COMMIT_MSG=$(git log --no-merges -1 --oneline)

# The commit marker "[cd build]" will trigger the build when required
if [[ "$COMMIT_MSG" =~ \[cd\ build\] ]]; then
    echo "::set-output name=build::true"
fi
