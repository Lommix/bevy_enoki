#! /bin/bash
enoki_tmp=$(mktemp -d)
editor_tmp=$(mktemp -d)

cp -r crates/enoki2d/* "$enoki_tmp"/.
cp -r LICENSE-MIT LICENSE-APACHE README.md "$enoki_tmp"/.

cp -r crates/enoki2d_editor/* "$editor_tmp"/.
cp -r LICENSE-MIT LICENSE-APACHE "$editor_tmp"/.

sed -i 's|"../../../README.md"|"../README.md"|g' "$enoki_tmp"/src/lib.rs

version=$(grep "^version" $enoki_tmp/Cargo.toml | sed 's/version = "\(.*\)"/\1/')
sed -i '/^bevy_enoki/c\bevy_enoki="'${version}'"' "$editor_tmp"/Cargo.toml

cd $enoki_tmp && cargo publish
cd $editor_tmp && cargo publish

rm -rf "$enoki_tmp"
rm -rf "$editor_tmp"
