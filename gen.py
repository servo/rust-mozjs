import argparse, subprocess;

bindgen = "bindgen";

jsapi = "../mozjs/js/src/jsapi.h"
includes = [
    "-I", "../mozjs/js/src/dist/include",
    ]
sysincludes = [
    "-isystem", "/usr/lib/x86_64-linux-gnu/gcc/x86_64-linux-gnu/4.5/include",
    "-isystem", "/usr/lib/gcc/x86_64-redhat-linux/4.7.0/include"
    ]

args = [
    bindgen,
    "-l", "mozjs",
    "-o", "jsapi.rs",
    "-match" ,"js",
    jsapi]
args += includes + sysincludes

subprocess.call(args)
        
# To generate jsglue:
# DYLD_LIBRARY_PATH=~/versioned/rust-mozilla/build/llvm/x86_64-apple-darwin/Release+Asserts/lib/ ~/versioned/rust-bindgen/bindgen ./jsglue.c -I ../../build/src/mozjs/dist/include/ -match glue > glue.rs
