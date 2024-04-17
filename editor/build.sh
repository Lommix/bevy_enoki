#!/bin/bash
cargo build --target wasm32-unknown-unknown --release

wasm-bindgen --no-typescript --target web --out-dir ./web --out-name "editor" ../target/wasm32-unknown-unknown/release/editor.wasm
wasm-opt -Oz --output ./web/editor_bg.wasm ./web/editor_bg.wasm
