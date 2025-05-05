#!/bin/sh

echo "all";

echo "cc=${CC},rustc=${RUSTC}"
echo "ALL=${ALL},TEST=${TEST},COVERAGE=${COVERAGE},CLEAN=${CLEAN}"

TOML=`${FAM_BASE}/bin/famtoml ${DIRECTORY}/fam.toml` || exit 1;

if [ ! -e "${DIRECTORY}/target" ]; then
        mkdir -p ${DIRECTORY}/target;
fi
echo ${TOML}
