#!/usr/bin/env bash
set -euo pipefail

# ── Read current version from Cargo.toml ─────────────────────────────────────
current=$(grep -m1 '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo "Current version: $current"

IFS='.' read -r major minor patch <<< "$current"

# ── Prompt for bump type ──────────────────────────────────────────────────────
echo ""
echo "Bump type:"
echo "  1) major  ($major.$minor.$patch → $((major+1)).0.0)"
echo "  2) minor  ($major.$minor.$patch → $major.$((minor+1)).0)"
echo "  3) patch  ($major.$minor.$patch → $major.$minor.$((patch+1)))"
echo ""
read -rp "Choice [1/2/3]: " choice

case "$choice" in
    1) new_version="$((major+1)).0.0" ;;
    2) new_version="$major.$((minor+1)).0" ;;
    3) new_version="$major.$minor.$((patch+1))" ;;
    *) echo "Invalid choice. Aborting."; exit 1 ;;
esac

echo ""
echo "Bumping $current → $new_version"
read -rp "Confirm? [y/N] " confirm
[[ "$confirm" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 1; }

# ── Bump version in Cargo.toml ────────────────────────────────────────────────
sed -i "0,/^version = \"$current\"/s//version = \"$new_version\"/" Cargo.toml

# Update Cargo.lock
cargo build -q 2>/dev/null || true

# ── Commit ────────────────────────────────────────────────────────────────────
git add Cargo.toml Cargo.lock
git commit -m "Bump version to $new_version"

# ── Tag ───────────────────────────────────────────────────────────────────────
git tag "v$new_version"

# ── Push ─────────────────────────────────────────────────────────────────────
git push origin HEAD
git push origin "v$new_version"

echo ""
echo "Released v$new_version — GitHub Actions will build and publish the release."
