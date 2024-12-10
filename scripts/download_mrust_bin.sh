#!/bin/sh

git clone https://github.com/cgilliard/rust-bins
./scripts/mrbuild.sh --mrustc=./rust-bins/macos/mrustc --output=./rust-bins/macos/output
