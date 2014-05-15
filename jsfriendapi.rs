/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

use jsapi::{JSContext, JSObject};

pub type JSJitPropertyOp = *u8;

pub struct JSJitInfo {
    pub op: JSJitPropertyOp,
    pub protoID: u32,
    pub depth: u32,
    pub isInfallible: bool,
    pub isConstant: bool
}

extern {
pub fn JS_ObjectToOuterObject(cx: *mut JSContext,
                              obj: *mut JSObject) -> *mut JSObject;
}

//pub type JSJitInfo = JSJitInfo_struct;

#[nolink]
pub mod bindgen {
    use jsapi::{JSContext, JSObject, JSClass, JSRuntime};
    use libc::uintptr_t;

    extern {
        pub fn JS_NewObjectWithUniqueType(cx: *JSContext, clasp: *JSClass,
                                          proto: *JSObject, parent: *JSObject) -> *JSObject;
        pub fn JS_GetAddressableObject(rt: *JSRuntime, candidateObj: uintptr_t) -> *JSObject;
    }
}
