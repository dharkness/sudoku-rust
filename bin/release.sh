#!/usr/bin/env sh
#
# Usage: bin/release.sh <version>

cargo build --release
cp -f target/release/sudoku-rust "sudoku-rust-linux-x86_64-$1"
strip "sudoku-rust-linux-x86_64-$1"

cargo build --release --target x86_64-pc-windows-gnu
mv target/x86_64-pc-windows-gnu/release/sudoku-rust.exe "sudoku-rust-windows-x86_64-$1.exe"
strip "sudoku-rust-windows-x86_64-$1.exe"
