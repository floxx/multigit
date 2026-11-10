#!/bin/sh
# Print the CHANGELOG.md section of the given version (without its header line).
# Fails when the section does not exist or is empty.
set -eu

version="$1"
changelog="${2:-CHANGELOG.md}"

notes=$(awk -v header="## [$version]" '
    $0 == header { found = 1; next }
    found && (/^## \[/ || /^---$/) { exit }
    found { print }
' "$changelog")

if [ -z "$(printf '%s' "$notes" | tr -d '[:space:]')" ]; then
    echo "No changelog section for version $version in $changelog" >&2
    exit 1
fi
printf '%s\n' "$notes"
