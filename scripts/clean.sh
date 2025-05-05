#!/bin/sh

if [ -e ${DIRECTORY}/target ]; then
	COMMAND="rm -rf ${DIRECTORY}/target/*";
	echo ${COMMAND};
	${COMMAND}
fi
