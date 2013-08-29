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

//pub type JSJitInfo = JSJitInfo_struct;

#[nolink]
pub mod bindgen {
    use jsapi::{JSContext, JSObject, JSClass};

    extern {
        pub fn JS_NewObjectWithUniqueType(cx: *JSContext, clasp: *JSClass,
                                          proto: *JSObject, parent: *JSObject) -> *JSObject;
    }
}
