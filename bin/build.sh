#!/usr/bin/env sh

cargo build --release
cp -f target/release/sudoku-rust .
