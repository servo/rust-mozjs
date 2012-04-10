import argparse, subprocess;

bindgen = "bindgen";

jsapi = "../mozjs/mozilla-central/js/src/jsapi.h"
includes = [
    "-I", "../mozjs/mozilla-central/js/src/dist/include"
    ]
sysincludes = [
    "-isystem", "/usr/lib/x86_64-linux-gnu/gcc/x86_64-linux-gnu/4.5/include",
    "-isystem", "/usr/lib/gcc/x86_64-linux-gnu/4.6/include"
    ]

args = [
    bindgen,
    "-l", "mozjs",
    "-o", "jsapi.rs",
    "-match" ,"js",
    jsapi]
args += includes + sysincludes

subprocess.call(args)
        
        
