/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */
pub type JSJitPropertyOp = *u8;

pub struct JSJitInfo {
    op: JSJitPropertyOp,
    protoID: u32,
    depth: u32,
    isInfallible: bool,
    isConstant: bool
}




#[nolink]
pub mod bindgen {
    use jsapi::{JSContext, JSObject, JSClass, jsid, JSBool};
    use std::libc::c_void;

    pub type struct_IdVector = c_void;
    pub type IdVector = struct_IdVector;

    pub static JSITER_ENUMERATE: u32 = 1_u32;
    pub static JSITER_OWNONLY: u32 = 8_u32;
    pub static JSITER_HIDDEN: u32 = 16_u32;

    extern {
        pub fn JS_NewObjectWithUniqueType(cx: *JSContext, clasp: *JSClass,
                                          proto: *JSObject, parent: *JSObject) -> *JSObject;

        pub fn JS_IdVectorAppend(vector: *IdVector, id: jsid) -> JSBool;

        pub fn JS_GetPropertyNames(cx: *JSContext, obj: *JSObject, flags: uint, props: *IdVector) -> bool;
    }

    pub fn INT_TO_JSID(i: i32) -> jsid
    {
        return ((i << 1) | 0x1) as jsid;
    }
}
