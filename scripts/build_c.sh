#!/bin/sh

if [ "${CC}" = "" ]; then
	echo "CC not set!";
	exit 1;
fi

for file in ${DIRECTORY}/c/*.c
do
	if [ -f "$file" ]; then
		BASENAME=$(basename "$file" .c);
		OBJ=${DIRECTORY}/target/objs/${BASENAME}.o
        	if [ ! -e ${OBJ} ] || [ ${file} -nt ${OBJ} ]; then
			COMMAND="${CC} ${CCFLAGS} -o ${OBJ} -c ${file}";
			echo ${COMMAND};
			${COMMAND} || exit 1;
        	fi
	fi
done
