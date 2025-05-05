#!/bin/sh

COUNT=`echo ${TOML} | cut -d ' ' -f 3`

i=1;
while [ "$i" -le ${COUNT} ]
do
	CRATE_INDEX=`expr 1 + $i \* 3`
	PATH_INDEX=`expr $CRATE_INDEX + 2`;
	CRATE_NAME=`echo ${TOML} | cut -d ' ' -f $CRATE_INDEX`;
	PATH_NAME=`echo ${TOML} | cut -d ' ' -f $PATH_INDEX`;
	echo "building dep crate $CRATE_NAME in '$PATH_NAME'";
	DEP_DIR=${DIRECTORY}/target/deps/$CRATE_NAME

	# copy files over if they don't exist
	if [ ! -e ${DEP_DIR} ]; then
		mkdir -p ${DEP_DIR}/c || exit 1;
		mkdir -p ${DEP_DIR}/rust || exit 1;

		# Check and copy C files
		C_SRC_DIR="${DIRECTORY}/${PATH_NAME}/c"
		C_DEST_DIR="${DEP_DIR}/c"
		if [ -d "$C_SRC_DIR" ]; then
			C_FILES=`ls $C_SRC_DIR 2>/dev/null`
			if [ -n "$C_FILES" ]; then
				COMMAND="cp -rp $C_SRC_DIR/* $C_DEST_DIR"
				echo "$COMMAND"
				${COMMAND} || exit 1;
			fi
		fi

		# Check and copy Rust files
		RUST_SRC_DIR="${DIRECTORY}/${PATH_NAME}/rust"
		RUST_DEST_DIR="${DEP_DIR}/rust"
		if [ -d "$RUST_SRC_DIR" ]; then
			RUST_FILES=`ls $RUST_SRC_DIR 2>/dev/null`
			if [ -n "$RUST_FILES" ]; then
				COMMAND="cp -rp $RUST_SRC_DIR/* $RUST_DEST_DIR"
				echo "$COMMAND"
				${COMMAND} || exit 1;
			fi
		fi
	fi

	PREV_DIRECTORY=${DIRECTORY}
	PREV_BIN=${BIN}
	DIRECTORY=${DEP_DIR}
	BIN=${CRATE_NAME}
	
	mkdir -p ${DIRECTORY}/target || exit 1;
	mkdir -p ${DIRECTORY}/target/bin || exit 1;
	mkdir -p ${DIRECTORY}/target/lib || exit 1;
	mkdir -p ${DIRECTORY}/target/deps || exit 1;
	mkdir -p ${DIRECTORY}/target/main || exit 1;
	mkdir -p ${DIRECTORY}/target/objs || exit 1;
	# build c
	. ${FAM_BASE}/scripts/build_c.sh "$@" || exit 1;
	# build rust
	. ${FAM_BASE}/scripts/build_rust.sh "$@" || exit 1;

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

	DIRECTORY=${PREV_DIRECTORY}
	BIN=${PREV_BIN}

	i=`expr $i + 1`
done
