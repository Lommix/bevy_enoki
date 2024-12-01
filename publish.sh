#! /bin/bash
tmp=$(mktemp -d)

echo "$tmp"

# enoki 2d
cp -r crates/enoki2d/* "$tmp"/.
cp -r LICENSE-MIT LICENSE-APACHE README.md "$tmp"/.
sed -i 's|"../../../README.md"|"../README.md"|g' "$tmp"/src/lib.rs

cd $tmp && cargo publish
rm -rf "$tmp"
