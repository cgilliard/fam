#!/bin/sh

cd secp256k1-zkp
./autogen.sh
./configure \
	--enable-module-schnorrsig \
	--enable-module-rangeproof \
	--enable-module-generator \
	--enable-experimental
make
cp .libs/libsecp256k1.a ../.obj
cd ..
