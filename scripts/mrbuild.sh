rm bin/fam
mrustc --crate-type=lib rust/mod.rs -L../mrustc/output
clang -c c/main.c
clang -c c/sys.c
clang -o bin/fam *.o
rm libmod.rlib*
rm *.o
