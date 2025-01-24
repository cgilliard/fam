#!/bin/sh

cd secp256k1-zkp
if [ ! -f "./configure" ]; then
	./autogen.sh
	./configure \
		--enable-module-schnorrsig \
		--enable-module-rangeproof \
		--enable-module-generator \
		--enable-module-musig \
		--enable-experimental
fi
make
cp .libs/libsecp256k1.a ../.obj
cd ..
