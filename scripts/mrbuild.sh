#!/bin/sh

rm bin/fam
mrustc --crate-type=lib rust/mod.rs -L../mrustc/output || exit -1;
clang -c c/main.c || exit -1;
clang -c c/sys.c || exit -1;
clang -o bin/fam *.o || exit -1;
rm libmod.rlib*
rm *.o
