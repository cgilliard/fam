#!/bin/sh

clang -c -Ic c/sys.c || exit 1;
ar rcs libtest.a *.o || exit 1;
rustc --test rust/mod.rs -C instrument-coverage -C opt-level=0 -o bin/test_fam -L . -l static=test || exit 1;
export LLVM_PROFILE_FILE="/tmp/file.profraw"
./bin/test_fam || exit 1;
grcov /tmp/file.profraw --branch --binary-path ./bin --llvm-path=/Users/christophergilliard/homebrew/opt/llvm/bin > /tmp/coverage.txt
rm -f libtest.a *.o
