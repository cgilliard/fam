#!/bin/sh

TOML=`${FAM_BASE}/bin/famtoml ${DIRECTORY}/fam.toml` || exit 1;
IS_BIN=0;
if [ "`echo ${TOML} | cut -d ' ' -f 1`" = "bin" ]; then
        IS_BIN=1;
fi
BIN=`echo ${TOML} | cut -d ' ' -f 2`;

mkdir -p ${DIRECTORY}/target || exit 1;
mkdir -p ${DIRECTORY}/target/out || exit 1;
mkdir -p ${DIRECTORY}/target/deps || exit 1;
mkdir -p ${DIRECTORY}/target/objs || exit 1;

DEP_PATH=${DIRECTORY}
DEST_BASE=${DIRECTORY}/target/deps
CYCLE_STACK_PATH=/tmp/cycle

FINAL_BIN=$BIN
FINAL_IS_BIN=$IS_BIN
FINAL_DIRECTORY=$DIRECTORY

DEPTH=0
. ${FAM_BASE}/scripts/dep.sh

DIRECTORY=${FINAL_DIRECTORY}
. ${FAM_BASE}/scripts/build_c.sh
. ${FAM_BASE}/scripts/build_rust.sh

if [ ${FINAL_IS_BIN} -eq 1 ]; then
	BINARY=${FINAL_DIRECTORY}/target/out/${FINAL_BIN}
	OBJ_FILES=${FINAL_DIRECTORY}/target/objs/*
	NEED_CC=0
        for obj in $OBJ_FILES
        do
                if [ -f "$obj" ]; then
                        if [ ! -e "$BINARY" ] || [ "$obj" -nt "$BINARY" ]; then
                                NEED_CC=1
                                break
                        fi
                fi
        done

	if [ ${NEED_CC} -eq 1 ]; then
        	COMMAND="${CC} -o ${BINARY} \
${OBJ_FILES} \
${FINAL_DIRECTORY}/target/deps/*/target/out/*.a"
        	echo ${COMMAND}
        	${COMMAND} || exit 1;
	fi
else
	ARCHIVE="${FINAL_DIRECTORY}/target/out/lib${FINAL_BIN}.a"
        AR=ar
        OBJ_FILES="${FINAL_DIRECTORY}/target/objs/*.o"
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
                COMMAND="${AR} rcs ${ARCHIVE} ${FINAL_DIRECTORY}/target/objs/*.o"
                echo ${COMMAND}
                ${COMMAND} || exit 1;
        fi
fi
