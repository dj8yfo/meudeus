#!/bin/bash

set -e

# $DIR preview
cargo install --locked exa
# $FILE preview
cargo install --locked bat

rm -rf /tmp/helix
pushd /tmp
git clone https://github.com/helix-editor/helix
pushd helix
# $FILE open with helix editor `hx`
cargo install --locked --path helix-term

popd; 
popd;
rm -rf /tmp/helix

# these two are used for opening $DIR in default config
cargo install --locked zellij
cargo install --locked broot

# this is where `wl-copy` from default config is found for piping $SNIPPET_TEXT into it
cargo install --locked  wl-clipboard-rs-tools
