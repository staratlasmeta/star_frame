#!/bin/bash

# Script to update the star_frame version in the template cargo_toml
# Takes the new version as the first argument

set -e  # Exit on any error

# Check if version is provided
if [ $# -eq 0 ]; then
    echo "Error: Version parameter is required"
    echo "Usage: $0 <version>"
    exit 1
fi

VERSION=$1

# Path to the template cargo_toml file
TEMPLATE_FILE="star_frame_cli/src/template/cargo_toml"

if [ ! -f "$TEMPLATE_FILE" ]; then
    echo "Error: Template file not found at $TEMPLATE_FILE" >&2
    exit 1
fi

# Update the star_frame version in the template
sed -i "s/star_frame = { version = \"[^\"]*\"/star_frame = { version = \"$VERSION\"/" "$TEMPLATE_FILE"

# Verify the change
if ! grep -q "star_frame = { version = \"$VERSION\"" "$TEMPLATE_FILE"; then
    echo "Error: Version update verification failed" >&2
    exit 1
fi

echo "Template updated to version $VERSION"
