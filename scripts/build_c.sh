#!/bin/sh

echo "building c: '$@'"

for file in ${DIRECTORY}/c/*.c
do
	if [ -f "$file" ]; then
	BASENAME=$(basename "$file" .c);
		OBJ=${DIRECTORY}/target/objs/c_${BASENAME}.o
        	if [ ! -e ${OBJ} ] || [ ${file} -nt ${OBJ} ]; then
			COMMAND="${CC} ${CCFLAGS} -o ${OBJ} -c ${file}";
			echo ${COMMAND};
			${COMMAND} || exit 1;
        	fi
	fi
done
