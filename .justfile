# print options
default:
    @just --list --unsorted

# install cargo tools
init:
    cargo upgrade --incompatible
    cargo update

# check code
check:
    cargo check
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features

# automatically fix clippy warnings
fix:
    cargo fmt --all
    cargo clippy --allow-dirty --allow-staged --fix

# build project
build:
   cargo build --all-targets

# execute tests
test:
   cargo test

# execute benchmarks
bench:
    cargo bench


# Release a new version of fixlite.
# Usage:
#   just release           # bump patch (default)
#   just release minor     # bump minor
#   just release major     # bump major
# Run every recipe line with Bash + strict flags
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

release ARG="patch":
    #!/usr/bin/env bash
    kind="{{ARG}}"

    # 1. Ensure the working tree is clean
    if [[ -n $(git status --porcelain) ]]; then
        echo "🚫 There are have uncommitted changes or untracked files in the workspace. Commit, stash or clean first." >&2
        exit 1
    fi

    # 2. Run the project checks
    just check

    # 3. Grab the current version from Cargo.toml
    current_version=$(grep -m1 '^version[[:space:]]*=' Cargo.toml |
                      sed -E 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/')
    echo "Current version: $current_version"

    # 4. Bump the semver according to ${kind}

    IFS='.' read -r major minor patch <<< "$current_version"
    case "${kind}" in
      major) major=$((major + 1)); minor=0; patch=0 ;;
      minor) minor=$((minor + 1)); patch=0          ;;
      patch|*) patch=$((patch + 1))                 ;;
    esac
    new_version="${major}.${minor}.${patch}"
    echo "🔖 Bumping $kind to: $new_version"

    # 5. Update both Cargo.toml files
    sed -i.bak -E \
        "s/^version[[:space:]]*=[[:space:]]*\"[0-9]+\.[0-9]+\.[0-9]+\"/version = \"$new_version\"/" \
        Cargo.toml \
        fixlite_derive/Cargo.toml

    # Update the `fixlite` dependency inside fixlite_derive
    sed -i.bak -E \
        "s/(fixlite[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*=[[:space:]]*\")([0-9]+\.[0-9]+\.[0-9]+)(\"[^}]*\})/\1$new_version\3/" \
        fixlite_derive/Cargo.toml
    rm -f *.bak

    # 6. Commit, tag, and push
    git add Cargo.toml fixlite_derive/Cargo.toml
    git commit -m "Release: v$new_version"
    git tag "v$new_version"
    git push origin --tags HEAD:main

    echo "✅ Released v$new_version"
