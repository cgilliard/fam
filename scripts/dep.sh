#!/bin/sh

# Input:
# ${DEP_PATH} or ${DEP_GIT} only one allowed - location of source project.
# ${DEST_BASE} - base directory where the deps are built.
# ${CYCLE_STACK_PATH} - path to the file used to track cycles.

# 1.) Calculate shasum of the fam.toml (this is used as the directory within ${DEST_BASE).
# 2.) If this directory already exists, we return without action (dependency already met).
# 3.) Check if any of our dependencies are in the ${CYCLE_STACK_PATH} file already, report error if so.
# 4.) Otherwise, we call this script recursively with the DEP_PATH/DEP_GIT variable set for each of this node's deps.
# This handles the dfs aspect by handling our own depdencies before ourself. We also append our node to the CYCLE_STACK_PATH file.
# 5.) Once our deps have been executed, we compile our own source files into our shasum hash subdirectory within the DEST_BASE
# and return control back to the calling script (resetting the input variables back to their original state so as to not mess up
# the caller's env variables).

if [ "${FAM_BASE}" = "" ]; then
	echo "FAM_BASE must be set"
	exit 1;
fi

# Currently only DEP_PATH supported
if [ "${DEP_PATH}" = "" ] || [ "${DEST_BASE}" = "" ] || [ "${CYCLE_STACK_PATH}" = "" ]; then
	echo "DEP_PATH, DEST_BASE, and CYCLE_STACK_PATH must be set";
	echo "${DEP_PATH}, ${DEST_BASE}, ${CYCLE_STACK_PATH}";
	exit 1;
fi

SHASUM=`shasum "${DEP_PATH}/fam.toml" | cut -d ' ' -f 1` || exit 1;

COPY=1
if [ -e ${DEST_BASE}/${SHASUM}/complete ]; then
	COPY=0;
fi

TOML=`${FAM_BASE}/bin/famtoml ${DEP_PATH}/fam.toml` || exit 1;

COUNT=`echo ${TOML} | cut -d ' ' -f 3`
i=1;

while [ "$i" -le ${COUNT} ]
do
	DEP_PATH_PREV=${DEP_PATH}
	SHASUM_PREV=${SHASUM}
	i_prev=${i}
	TOML_PREV=${TOML}
	COUNT_PREV=${COUNT}

	CRATE_INDEX=`expr 1 + $i \* 3`
	PATH_INDEX=`expr $CRATE_INDEX + 2`;
	CRATE_NAME=`echo ${TOML} | cut -d ' ' -f $CRATE_INDEX`;
	NEXT_PATH=`echo ${TOML} | cut -d ' ' -f $PATH_INDEX`;
	DEP_PATH=${DEP_PATH}/${NEXT_PATH}
	DEPTH=`expr $DEPTH + 1`
	. ${FAM_BASE}/scripts/dep.sh
	DEPTH=`expr $DEPTH - 1`

	DEP_PATH=${DEP_PATH_PREV}
	SHASUM=${SHASUM_PREV}
	i=${i_prev}
	TOML=${TOML_PREV}
	COUNT=${COUNT_PREV}

	i=`expr $i + 1`
done

TARGET=${DEST_BASE}/${SHASUM}
mkdir -p ${TARGET}
mkdir -p ${TARGET}/c || exit 1;
mkdir -p ${TARGET}/rust || exit 1;
mkdir -p ${TARGET}/target || exit 1;
mkdir -p ${TARGET}/target/lib || exit 1;
mkdir -p ${TARGET}/target/objs || exit 1;

# Check and copy C files
if [ -d "${DEP_PATH}/c" ]; then
	C_FILES=`ls ${DEP_PATH}/c 2>/dev/null`
	if [ -n "$C_FILES" ]; then
		COMMAND="cp -rp ${DEP_PATH}/c/* ${TARGET}/c"
		${COMMAND} || exit 1;
	fi
fi

# Check and copy Rust files
if [ -d "${DEP_PATH}/rust" ]; then
	RUST_FILES=`ls ${DEP_PATH}/rust 2>/dev/null`
	if [ -n "$RUST_FILES" ]; then
		COMMAND="cp -rp ${DEP_PATH}/rust/* ${TARGET}/rust"
		${COMMAND} || exit 1;
	fi
fi

DIRECTORY=${TARGET}
if [ $DEPTH -ne 0 ]; then
	. ${FAM_BASE}/scripts/build_c.sh
	. ${FAM_BASE}/scripts/build_rust.sh
fi

IS_BIN=0;
if [ "`echo ${TOML} | cut -d ' ' -f 1`" = "bin" ]; then
        IS_BIN=1;
fi
BIN=`echo ${TOML} | cut -d ' ' -f 2`;


if [ $DEPTH -ne 0 ]; then
	ARCHIVE="${DIRECTORY}/target/lib/lib${BIN}.a"
	AR=ar
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

touch ${DEST_BASE}/${SHASUM}/complete
