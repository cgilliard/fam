#!/bin/sh

rm bin/fam
rustc -C opt-level=3 -C panic=abort --crate-type=lib -o rust.o rust/mod.rs || exit 1;
clang -c c/main.c || exit 1;
clang -c c/sys.c || exit 1;
clang -o bin/fam *.o || exit 1;
rm *.o
