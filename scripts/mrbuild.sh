#!/bin/sh

mrustc=mrustc
output=../mrustc/output
for var in "$@"; do
        case "$var" in
        --mrustc=*)
                mrustc=${var#*=}
                ;;
	--output=*)
		output=${var#*=}
		;;
	esac
done

echo "output='${output}'";

rm -f bin/fam
${mrustc} --crate-type=lib rust/mod.rs -L${output} || exit 1;
clang -c c/main.c || exit 1;
clang -c c/sys.c || exit 1;
clang -o bin/fam *.o || exit 1;
rm -f libmod.rlib*
rm -f *.o
