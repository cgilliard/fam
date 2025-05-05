#!/bin/sh

AR=ar

TOML=`${FAM_BASE}/bin/famtoml ${DIRECTORY}/fam.toml` || exit 1;
IS_BIN=0;
if [ "`echo ${TOML} | cut -d ' ' -f 1`" = "bin" ]; then
	IS_BIN=1;
fi
BIN=`echo ${TOML} | cut -d ' ' -f 2`;

mkdir -p ${DIRECTORY}/target || exit 1;
mkdir -p ${DIRECTORY}/target/bin || exit 1;
mkdir -p ${DIRECTORY}/target/lib || exit 1;
mkdir -p ${DIRECTORY}/target/deps || exit 1;
mkdir -p ${DIRECTORY}/target/main || exit 1;
mkdir -p ${DIRECTORY}/target/objs || exit 1;

. ${FAM_BASE}/scripts/build_deps.sh "$@" || exit 1;
. ${FAM_BASE}/scripts/build_c.sh "$@" || exit 1;
. ${FAM_BASE}/scripts/build_rust.sh "$@" || exit 1;


if [ ${IS_BIN} -eq 1 ]; then
	COMMAND="${CC} -o ${DIRECTORY}/target/bin/${BIN} ${DIRECTORY}/target/objs/* ${DIRECTORY}/target/deps/*/target/lib/*.a"
	echo ${COMMAND}
	${COMMAND} || exit 1;
else
	ARCHIVE="${DIRECTORY}/target/lib/lib${BIN}.a"
	OBJ_FILES="${DIRECTORY}/target/objs/*.o"
	NEED_AR=0

	# Test if any .o files exist
	for obj in $OBJ_FILES
	do
    		if [ -f "$obj" ]; then
        		# At least one object file exists
        		if [ ! -e "$ARCHIVE" ] || [ "$obj" -nt "$ARCHIVE" ]; then
            			NEED_AR=1
            			break
        		fi
    		fi
	done

	if [ "$NEED_AR" = "1" ]; then
		COMMAND="${AR} rcs ${ARCHIVE} ${DIRECTORY}/target/objs/*.o"
		echo ${COMMAND}
		${COMMAND} || exit 1;
	fi
fi
