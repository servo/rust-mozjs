#!/bin/bash

set -e

# Make sure we're in the toplevel rust-mozjs dir
cd "`dirname $0`"/..

if [[ ! -f "Cargo.toml" || ! -f "./etc/bindings.sh" ]] ; then
    echo "Expected to be in the toplevel rust-bindgen dir, but somehow in: `pwd`"
    exit 1
fi

if [[ ! -f ../rust-bindgen/target/debug/bindgen.exe ]] ; then
    echo "Can't find bindgen.exe in ../rust-bindgen/target/debug"
    exit 1
fi

if [[ "$VSINSTALLDIR" == "" ]] ; then
    echo "Visual Studio 2015 environment variables and paths must be set before running this!"
    exit 1
fi

handle_error() {
    set +x
    local parent_lineno="$1"
    local message="$2"
    local code="${3:-1}"
    if [[ -n "$message" ]] ; then
	echo "Error on or near line ${parent_lineno}: ${message}; exiting with status ${code}"
    else
	echo "Error on or near line ${parent_lineno}; exiting with status ${code}"
    fi
    exit "${code}"
}

trap 'handle_error ${LINENO}' ERR

build_bindings() {
    cargo clean
    cargo build $3 || echo ... ignoring first build error ...
    ./etc/bindings.sh $4
    cp out.rs src/jsapi_$1.rs
    cargo $2 $3
}

# Unset these, to make sure nothing wants to build for msvc
echo "Saving and clearing Visual Studio environment variables..."
saved_vsinstalldir="$VSINSTALLDIR"
saved_include="$INCLUDE"
saved_lib="$LIB"
unset VSINSTALLDIR
unset INCLUDE
unset LIB

set -x

# gnu first
rustup default nightly-x86_64-pc-windows-gnu

build_bindings windows_gcc_64 test ""

# We don't test debugmozjs; it takes way too long
build_bindings windows_gcc_64_debug build "--features debugmozjs"

set +x

# MSVC next

echo "Restoring Visual Studio variables..."
export VSINSTALLDIR="${saved_vsinstalldir}"
export INCLUDE="${saved_include}"
export LIB="${saved_lib}"

set -x

rustup default nightly-x86_64-pc-windows-msvc

build_bindings windows_msvc14_64 test "" msvc14

# We don't test debugmozjs; it takes way too long
build_bindings windows_msvc14_64_debug build "--features debugmozjs" msvc14

set +x

echo "==== Success! ===="

rm out.rs
