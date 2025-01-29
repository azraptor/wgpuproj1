#!/bin/sh
cargo build --release
wasm-pack build --target web