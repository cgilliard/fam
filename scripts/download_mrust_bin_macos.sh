#!/bin/sh

git clone https://github.com/cgilliard/rust-bins
./fam --mrustc --with-mrustc=./rust-bins/macos/mrustc --output=./rust-bins/macos/output
