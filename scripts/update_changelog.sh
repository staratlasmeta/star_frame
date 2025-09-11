#!/bin/bash

# Update CHANGELOG.md for a new release
# Takes the new version as the first argument

set -e  # Exit on any error

# Check if version is provided
if [ $# -eq 0 ]; then
    echo "Error: Version parameter is required"
    echo "Usage: $0 <version>"
    exit 1
fi

VERSION=$1
DATE=$(date +%Y-%m-%d)

# Path to the CHANGELOG.md file
CHANGELOG_FILE="CHANGELOG.md"

if [ ! -f "$CHANGELOG_FILE" ]; then
    echo "Error: CHANGELOG.md not found at $CHANGELOG_FILE" >&2
    exit 1
fi

# Get the previous version from the changelog for link generation
PREVIOUS_VERSION=$(grep "^## \[" "$CHANGELOG_FILE" | head -2 | tail -1 | sed 's/.*\[\([0-9.]*\)\].*/\1/')

if [ -z "$PREVIOUS_VERSION" ]; then
    PREVIOUS_VERSION="HEAD"
fi

# Add new version section after [Unreleased]
# This replaces the [Unreleased] line with itself + blank line + new version section
sed -i "s|^## \[Unreleased\]$|## [Unreleased]\\
\\
## [$VERSION] - $DATE|" "$CHANGELOG_FILE"

# Add the new version link at the bottom
sed -i "/^\[unreleased\]:/a\\[$VERSION]: https://github.com/staratlasmeta/star_frame/compare/v$PREVIOUS_VERSION...v$VERSION" "$CHANGELOG_FILE"

# Update the [unreleased] link to compare from the new version
sed -i "s|\[unreleased\]:.*|\[unreleased\]: https://github.com/staratlasmeta/star_frame/compare/v$VERSION...HEAD|" "$CHANGELOG_FILE"

# Verify the change
if ! grep -q "## \[$VERSION\] - $DATE" "$CHANGELOG_FILE"; then
    echo "Error: CHANGELOG.md update verification failed" >&2
    exit 1
fi

echo "CHANGELOG.md updated for version $VERSION"
