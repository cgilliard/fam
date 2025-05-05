#!/bin/sh

echo "building rust: '${DIRECTORY}'"

NEED_UPDATE=0;
if [ -e ${DIRECTORY}/rust ]; then
	for file in `find ${DIRECTORY}/rust | grep "\.rs$"`
	do
		OBJ=${DIRECTORY}/target/objs/rust_${BIN}.o
		if [ ! -e ${OBJ} ] || [ ${file} -nt ${OBJ} ]; then
			NEED_UPDATE=1;
			break;
		fi
	done
fi

if [ ${NEED_UPDATE} -eq 1 ]; then
	if [ -f ${DIRECTORY}/rust/lib.rs ]; then
		COMMAND="${RUSTC} \
-C panic=abort \
--crate-name=${BIN} \
--crate-type=staticlib \
-o ${DIRECTORY}/target/objs/rust_${BIN}.o \
--emit=obj
${DIRECTORY}/rust/lib.rs"
		echo ${COMMAND}
		${COMMAND} || exit 1;
	fi
fi
