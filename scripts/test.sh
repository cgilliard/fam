#!/bin/sh



rm -f libtest.a *.o
clang -c -Ic c/sys.c || exit 1;
clang -c -Ic c/util.c || exit 1;
ar rcs libtest.a *.o || exit 1;
rustc +nightly --test rust/mod.rs -o bin/test_fam -L . -l static=test || exit 1;
./bin/test_fam $1 || exit 1;
rm -f libtest.a *.o
