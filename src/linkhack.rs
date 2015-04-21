/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

// Some crumminess to make sure we link correctly

#[cfg(target_os = "linux")]
#[link(name = "pthread")]
#[link(name = "js_static", kind = "static")]
#[link(name = "mozglue", kind = "static")]
#[link(name = "stdc++")]
#[link(name = "z")]
extern { }

#[cfg(target_os = "macos")]
#[link(name = "js_static", kind = "static")]
#[link(name = "mozglue", kind = "static")]
#[link(name = "stdc++")]
#[link(name = "z")]
extern { }

// Avoid hard linking with stdc++ in android ndk cross toolchain so that
// the ELF header will have an entry for libstdc++ and we will know to
// open it explicitly.
//It is hard to find location of android system libs in this rust source file
//and also we need to size down for mobile app packaging
#[cfg(target_os = "android")]
#[link(name = "mozjs")]
#[link_args="-lstdc++"]
#[link(name = "z")]
extern { }
