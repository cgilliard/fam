#!/bin/sh

# Main suboptions
ALL=0
TEST=0
COVERAGE=0
CLEAN=0
DIRECTORY="."

export CC=clang
export RUSTC=famc
COUNT=0;

for arg in "$@"; do
	case "$arg" in
		test)
			if [ ${COUNT} -ne 0 ]; then
				echo "Unexpected token: '$arg'";
				exit 1;
			fi
			TEST=1
		;;
		coverage)
			if [ ${COUNT} -ne 0 ]; then
				echo "Unexpected token: '$arg'";
                                exit 1;
                        fi      
			COVERAGE=1
                ;;
                clean)
                        if [ ${COUNT} -ne 0 ]; then
                                echo "Unexpected token: '$arg'";
                                exit 1;
                        fi
                        CLEAN=1
                ;;
		all)
                        if [ ${COUNT} -ne 0 ]; then
                                echo "Unexpected token: '$arg'";
                                exit 1;
                        fi
                        ALL=1
                ;;
		--cc=*)
			export CC=${arg#*=};
			if [ -z "${CC}" ]; then
                		echo "Error: --cc requires a non-empty value: --cc=cc" >&2
				exit 1;
			fi
		;;
		--cc)
			echo "Error: --cc requires a non-empty value: --cc=cc" >&2
			exit 1;
		;;
		--rustc=*)
			export RUSTC=${arg#*=};
			if [ -z "${RUSTC}" ]; then
				echo "Error: --rustc requires a non-empty value: --rustc=famc" >&2
				exit 1;
			fi
		;;
		--rustc)
			echo "Error: --rustc requires a non-empty value: --rustc=famc" >&2
			exit 1;
		;;
		-d=*)
			DIRECTORY=${arg#*=};
			if [ -z "${DIRECTORY}" ]; then
                                echo "Error: -d requires a non-empty value: -d=/path/to/project" >&2
                                exit 1;
                        fi
		;;
		-d)
			echo "Error: -d requires a non-empty value: -d=/path/to/project" >&2
                        exit 1;
		;;
		*)
			echo "ERROR: Unknown option: $arg" >&2
			exit 1;
		;;
	esac
	COUNT=$(expr $COUNT + 1)
done

if [ ${CLEAN} -eq 0 ] && [ ${TEST} -eq 0 ] && [ ${COVERAGE} -eq 0 ]; then
	ALL=1
fi


