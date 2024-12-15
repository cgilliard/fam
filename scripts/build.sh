#!/bin/sh

rm -f bin/fam *.o
rustc +nightly -C opt-level=3 --emit=obj --crate-type=lib -o rust.o rust/mod.rs || exit 1;
clang -c -Ic c/main.c || exit 1;
clang -c -Ic c/sys.c || exit 1;
clang -c -Ic c/util.c || exit 1;
clang -o bin/fam *.o || exit 1;
rm *.o
