/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

pub type JSJitPropertyOp = *u8;

pub struct JSJitInfo {
    pub op: JSJitPropertyOp,
    pub protoID: u32,
    pub depth: u32,
    pub isInfallible: bool,
    pub isConstant: bool
}

//pub type JSJitInfo = JSJitInfo_struct;

pub mod bindgen {
    use jsapi::{JSContext, JSObject, JSClass, JSRuntime, JSHandleObject};
    use libc::uintptr_t;

    extern {
        pub fn JS_NewObjectWithUniqueType(cx: *mut JSContext, clasp: *JSClass,
                                          proto: JSHandleObject, parent: JSHandleObject) -> *mut JSObject;
        pub fn JS_GetAddressableObject(rt: *mut JSRuntime, candidateObj: uintptr_t) -> *mut JSObject;
    }
}
