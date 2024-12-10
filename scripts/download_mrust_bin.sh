#!/bin/sh

git clone https://github.com/cgilliard/rust-bins
./scrripts/mrbuild.sh --mrustc=./rust-bins/macos/mrustc --output=./rust-bins/macos/output
