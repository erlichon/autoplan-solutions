#!/bin/sh
set -e

binary_name="${1:?missing argument: which binary to build}"

tar="tar"
if [ "$(uname)" = "Darwin" ]; then
    tar="gtar --no-xattrs"
fi

archive_path="$(pwd -P)-$binary_name.tar.xz"

tmpdir="$(mktemp -d)"
echo "$tmpdir"
git worktree add --detach "$tmpdir/repo"
cd "$tmpdir/repo"

    # remove other binaries
    mv src/bin/"$binary_name".rs ../"$binary_name".rs
    rm -r src/bin
    mkdir src/bin
    mv ../"$binary_name".rs src/bin/"$binary_name".rs

    # remove other junk
    rm -rf scripts
    rm -rf spec
    rm -rf tests
    rm -rf .vscode
    rm -rf docs

    # remove dev dependencies
    sed -i.bak '/\[dev-dependencies\]/,$d' Cargo.toml
    rm Cargo.toml.bak

    # lock vendors
    cargo vendor
    mkdir .cargo
    printf '%s\n' \
        '[source.crates-io]' \
        'replace-with = "vendored-sources"' \
        '[source.vendored-sources]' \
        'directory = "vendor"' \
        > .cargo/config.toml

    # create archive
    $tar --exclude-vcs --exclude="./*.DS_Store" -cavf "$archive_path" .

cd -
git worktree remove -f "$tmpdir/repo"
rmdir "$tmpdir"
