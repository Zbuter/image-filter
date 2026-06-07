#!/bin/bash

# Generate changelog from git commits since last tag
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

if [ -z "$LAST_TAG" ]; then
    echo "No previous tags found. Showing all commits."
    COMMITS=$(git log --oneline)
else
    echo "Generating changelog since $LAST_TAG"
    COMMITS=$(git log ${LAST_TAG}..HEAD --oneline)
fi

echo ""
echo "## Commits:"
echo "$COMMITS"
echo ""
echo "Copy the relevant changes to CHANGELOG.md under the new version section."
