rm bin/fam
rustc -C panic=abort --crate-type=lib -o rust.o rust/mod.rs
clang -c c/main.c
clang -c c/sys.c
clang -o bin/fam *.o
rm *.o
