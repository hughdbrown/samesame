#!/bin/sh

cargo build --release
cp target/release/samesame /usr/local/bin/.

