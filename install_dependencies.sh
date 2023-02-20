#!/bin/bash

set -e

cargo install --locked exa
cargo install --locked bat

rm -rf /tmp/helix
pushd /tmp
git clone https://github.com/helix-editor/helix
pushd helix
cargo install --locked --path helix-term

popd; 
popd;
rm -rf /tmp/helix
cargo install --locked zellij
cargo install --locked broot
