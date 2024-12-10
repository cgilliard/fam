#!/bin/sh

git clone https://github.com/cgilliard/rust-bins
./scripts/mrbuild.sh --mrustc=./rust-bins/linux/mrustc --output=./rust-bins/linux/output
