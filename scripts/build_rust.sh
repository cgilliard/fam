#!/bin/sh

if [ "${RUSTC}" = "" ]; then
        echo "RUSTC not set!";
        exit 1;
fi

BIN=_rust_binary__
CRATE_NAME=`echo ${TOML} | cut -d ' ' -f 2`;

NEED_UPDATE=0;
if [ -e ${DIRECTORY}/rust ]; then
	for file in `find ${DIRECTORY}/rust | grep "\.rs$"`
	do
		OBJ=${DIRECTORY}/target/objs/${BIN}.o
		if [ ! -e ${OBJ} ] || [ ${file} -nt ${OBJ} ]; then
			NEED_UPDATE=1;
			break;
		fi
	done
fi

if [ ${NEED_UPDATE} -eq 1 ]; then
	if [ -f ${DIRECTORY}/rust/lib.rs ]; then
		if ${RUSTC} --version | grep -q "mrustc"; then
			EMIT=""
			EXT=""
		else
			EMIT="--emit obj"
			EXT=".o"
		fi

		COMMAND="${RUSTC} \
-C panic=abort \
--crate-name=${CRATE_NAME} \
${EMIT} \
--crate-type=lib \
${RUSTEXTRA} \
-o ${DIRECTORY}/target/objs/${BIN}${EXT} \
${DIRECTORY}/rust/lib.rs"
		echo ${COMMAND}
		${COMMAND} || exit 1;
	fi
fi
