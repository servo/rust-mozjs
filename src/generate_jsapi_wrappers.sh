#!/bin/sh
# This is one big heuristic but seems to work well enough
grep -v "link_name" jsapi_linux_64.rs | \
    grep -v '"\]' | \
    grep -F -v '/\*\*' | \
    sed -z 's/,\n */, /g' | \
    sed -z 's/:\n */: /g' | \
    sed -z 's/\n *->/ ->/g' | \
    grep -v '^\}$' | \
    sed 's/^ *pub/pub/' | \
    sed -z 's/\;\n/\n/g' | \
    grep 'pub fn' | \
    grep Handle | \
    grep -v Handler | \
    sed 's/Handle<\*mut JSObject>/HandleObject/g' | \
    sed 's/\(.*\)/wrap!(\1);/g' \
    > jsapi_wrappers.in
