#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Elena Gantner
#
# Bump the app version across all package.json and Cargo.toml files.
#
# Usage:
#   ./scripts/bump-version.sh <new-version>
#   ./scripts/bump-version.sh 0.3.0
#
# This updates:
#   - apps/desktop/package.json  (source of truth for Tauri via tauri.conf.json)
#   - apps/web/package.json
#   - Cargo.toml                 (workspace version)

set -euo pipefail

# Platform and dependency checks
if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "Error: This script is currently only supported on macOS."
  exit 1
fi

if ! command -v gsed &>/dev/null; then
  echo "Error: gsed is required. Please install coreutils via brew:"
  echo "  brew install coreutils"
  exit 1
fi

if ! command -v sponge &>/dev/null; then
  echo "Error: sponge is required. Please install moreutils via brew:"
  echo "  brew install moreutils"
  exit 1
fi

if [ $# -ne 1 ]; then
  echo "Usage: $0 <new-version>"
  echo "Example: $0 0.3.0"
  exit 1
fi

NEW_VERSION="$1"

# Validate semver format (basic check)
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'; then
  echo "Error: '$NEW_VERSION' is not a valid semver version (expected X.Y.Z)"
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Read current version from the source of truth
CURRENT_VERSION=$(jq -r .version "$ROOT_DIR/apps/desktop/package.json")
echo "Bumping version: $CURRENT_VERSION -> $NEW_VERSION"

# 1. apps/desktop/package.json (source of truth; tauri.conf.json reads from this)
jq --arg v "$NEW_VERSION" '.version = $v' "$ROOT_DIR/apps/desktop/package.json" | sponge "$ROOT_DIR/apps/desktop/package.json"
echo "  Updated apps/desktop/package.json"

# 2. apps/web/package.json
jq --arg v "$NEW_VERSION" '.version = $v' "$ROOT_DIR/apps/web/package.json" | sponge "$ROOT_DIR/apps/web/package.json"
echo "  Updated apps/web/package.json"

# 3. Cargo.toml workspace version
gsed -i "s/^version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/" "$ROOT_DIR/Cargo.toml"
echo "  Updated Cargo.toml (workspace)"

echo ""
echo "Done! Version is now $NEW_VERSION everywhere."
echo ""
echo "Files updated:"
echo "  - apps/desktop/package.json"
echo "  - apps/web/package.json"
echo "  - Cargo.toml (workspace)"
