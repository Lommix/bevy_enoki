#!/bin/bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --no-typescript --target web --out-dir ./web --out-name "editor" ../target/wasm32-unknown-unknown/release/editor.wasm
wasm-opt -Oz --output ./web/editor_bg.wasm ./web/editor_bg.wasm

cd ./web
gzip -c editor.js > editor.js.gz
gzip -c editor_bg.wasm > editor_bg.wasm.gz

scp *.gz lommix@lommix.de:/home/lommix/blog/wasm/particle/.
scp main.js lommix@lommix.de:/home/lommix/blog/wasm/particle/.
scp index.html lommix@lommix.de:/home/lommix/blog/wasm/particle/.
