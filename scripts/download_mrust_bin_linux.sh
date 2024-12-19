#!/bin/sh

git clone https://github.com/cgilliard/rust-bins
./fam --mrustc --with-mrustc=./rust-bins/linux/mrustc --output=./rust-bins/linux/output
