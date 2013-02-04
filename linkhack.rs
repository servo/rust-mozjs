/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

// Some crumminess to make sure we link correctly

#[cfg(target_os = "linux")]
#[link_args = "-lpthread -L. -ljs_static -lstdc++ -lz"]
#[nolink]
extern { }

#[cfg(target_os = "macos")]
#[link_args = "-L. -ljs_static -lstdc++ -lz"]
#[nolink]
extern { }

//Avoid hard linking with stdc++ in android ndk cross toolchain
//It is hard to find location of android system libs in this rust source file
//and also we need to size down for mobile app packaging
#[cfg(target_os = "android")]
#[link_args = "-L. -lmozjs -lstdc++ -lz"]
#[nolink]
extern { }
