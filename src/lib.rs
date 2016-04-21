/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "js"]
#![crate_type = "rlib"]

#![feature(core_intrinsics)]
#![feature(link_args)]
#![feature(unsafe_no_drop_flag)]
#![feature(const_fn)]

#![allow(drop_with_repr_extern)]
#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, improper_ctypes)]

#[macro_use]
extern crate heapsize;
extern crate libc;
#[macro_use]
extern crate log;
extern crate mozjs_sys;
extern crate num_traits;
extern crate rustc_serialize as serialize;

#[cfg(target_os = "linux")]
#[cfg(target_pointer_width = "64")]
mod jsapi_linux_64;

#[cfg(target_os = "macos")]
#[cfg(target_pointer_width = "64")]
mod jsapi_macos_64;

#[cfg(target_os = "windows")]
#[cfg(target_pointer_width = "64")]
mod jsapi_windows_gcc_64;

#[cfg(not(target_os = "windows"))]
#[cfg(target_pointer_width = "32")]
mod jsapi_linux_32;

pub mod jsapi {
    #[cfg(target_os = "linux")]
    #[cfg(target_pointer_width = "64")]
    pub use jsapi_linux_64::*;

    #[cfg(target_os = "macos")]
    #[cfg(target_pointer_width = "64")]
    pub use jsapi_macos_64::*;

    #[cfg(target_os = "windows")]
    #[cfg(target_pointer_width = "64")]
    pub use jsapi_windows_gcc_64::*;

    #[cfg(not(target_os = "windows"))]
    #[cfg(target_pointer_width = "32")]
    pub use jsapi_linux_32::*;
}

mod consts;
pub mod conversions;
pub mod error;
pub mod glue;
pub mod jsval;
pub mod rust;

pub use consts::*;

use heapsize::HeapSizeOf;
use jsapi::{JSContext, Heap};
use jsval::JSVal;
use rust::GCMethods;

#[inline(always)]
pub unsafe fn JS_ARGV(_cx: *mut JSContext, vp: *mut JSVal) -> *mut JSVal {
    vp.offset(2)
}

#[inline(always)]
pub unsafe fn JS_CALLEE(_cx: *mut JSContext, vp: *mut JSVal) -> JSVal {
    *vp
}

// This is measured properly by the heap measurement implemented in SpiderMonkey.
impl<T: Copy + GCMethods<T>> HeapSizeOf for Heap<T> {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}
known_heap_size!(0, JSVal);

