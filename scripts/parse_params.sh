#!/bin/sh

usage="Usage: fam [ all | test | fasttest | coverage ] [options]";

for var in "$@"; do
	case "$var" in
	-s)
		s=1
		;;
	--mrustc)
		usemrustc=1
		;;
	--filter=*)
		filter=${var#*=}
		;;
	all)
		all=1;
		ccflags=-O3
		;;
	output)
		output=${var#*=}
		;;
	fasttest)
		fasttest=1;
		ccflags=-O3
		;;
	--with-cc=*)
                cc=${var#*=}
                ;;
	--with-mrustc=*)
		mrustc=${var#*=}
		;;
	coverage)
		coverage=1;
		;;
	test)
		test=1;
		;;
	*)
		echo "Unrecognized option: '$var'"
		echo $usage;
		exit;
	esac
done

if [ "$test" != "1" ]  && [ "$coverage" != "1" ] && [ "$fasttest" != "1" ]; then
	all=1;
fi

