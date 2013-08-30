/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::libc;
use jsapi;
use jsapi::*;

pub extern fn JS_PropertyStub(cx: *JSContext, obj: JSHandleObject, id: JSHandleId, vp: JSMutableHandleValue) -> JSBool {
    unsafe {
        jsapi::JS_PropertyStub(cx, obj, id, vp)
    }
}

pub extern fn JS_StrictPropertyStub(cx: *JSContext, obj: JSHandleObject, id: JSHandleId, strict: JSBool, vp: JSMutableHandleValue) -> JSBool {
    unsafe {
        jsapi::JS_StrictPropertyStub(cx, obj, id, strict, vp)
    }
}

pub extern fn JS_EnumerateStub(cx: *JSContext, obj: JSHandleObject) -> JSBool {
    unsafe {
        jsapi::JS_EnumerateStub(cx, obj)
    }
}

pub extern fn JS_ResolveStub(cx: *JSContext, obj: JSHandleObject, id: JSHandleId) -> JSBool {
    unsafe {
        jsapi::JS_ResolveStub(cx, obj, id)
    }
}

pub extern fn JS_ConvertStub(cx: *JSContext, obj: JSHandleObject, _type: JSType, vp: JSMutableHandleValue) -> JSBool {
    unsafe {
        jsapi::JS_ConvertStub(cx, obj, _type, vp)
    }
}

pub extern fn JS_ArrayIterator(cx: *JSContext, argc: libc::c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        jsapi::JS_ArrayIterator(cx, argc, &*vp)
    }
}

