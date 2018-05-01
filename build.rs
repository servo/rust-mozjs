/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate cc;

use std::env;

fn main() {
    let mut build = cc::Build::new();
    build
        .file("src/jsglue.cpp")
        .include(&format!("{}/dist/include", env::var("DEP_MOZJS_OUTDIR").unwrap()));
    if env::var("CARGO_FEATURE_DEBUGMOZJS").is_ok() {
        build.define("DEBUG", "");
        build.define("_DEBUG", "");

        if cfg!(target_os = "windows") {
            build.flag("-MDd");
            build.flag("-Od");
        } else {
            build.flag("-g");
            build.flag("-O0");
        }
    } else if cfg!(target_os = "windows") {
        build.flag("-MD");
    }

    build.flag_if_supported("-Wno-c++0x-extensions");
    build.flag_if_supported("-Wno-return-type-c-linkage");
    build.flag_if_supported("-Wno-invalid-offsetof");

    let confdefs_path = format!("{}/js/src/js-confdefs.h", env::var("DEP_MOZJS_OUTDIR").unwrap());
    if cfg!(target_os = "windows") {
        build.flag(&format!("-FI{}", confdefs_path));
        build.define("WIN32", "");
        build.flag("-Zi");
        build.flag("-GR-");
    } else {
        build.flag("-fPIC");
        build.flag("-fno-rtti");
        build.flag("-std=c++11");
        build.define("JS_NO_JSVAL_JSID_STRUCT_TYPES", "");
        build.flag("-include");
        build.flag(&confdefs_path);
    }

    build.compile("jsglue");
    println!("cargo:rerun-if-changed=src/jsglue.cpp");
}
