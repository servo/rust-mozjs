/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg(feature = "debugmozjs")]

#[macro_use]
extern crate mozjs;
extern crate libc;

use mozjs::jsapi::CompartmentOptions;
use mozjs::jsapi::JSAutoCompartment;
use mozjs::jsapi::JSClass;
use mozjs::jsapi::JSContext;
use mozjs::jsapi::JSFunctionSpec;
use mozjs::jsapi::JS_GetObjectPrototype;
use mozjs::jsapi::JSNativeWrapper;
use mozjs::jsapi::JS_NewGlobalObject;
use mozjs::jsapi::JS_NewObjectWithUniqueType;
use mozjs::jsapi::JSPROP_ENUMERATE;
use mozjs::jsapi::JS_SetGCZeal;
use mozjs::jsapi::OnNewGlobalHookOption;
use mozjs::jsapi::Value;
use mozjs::jsval::JSVal;
use mozjs::rust::{Runtime, SIMPLE_GLOBAL_CLASS, define_methods};
use std::ptr;

#[test]
fn rooting() {
    unsafe {
        let runtime = Runtime::new().unwrap();
        let cx = runtime.cx();
        JS_SetGCZeal(cx, 2, 1);

        let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
        let c_option = CompartmentOptions::default();

        rooted!(in(cx) let global = JS_NewGlobalObject(cx,
                                                       &SIMPLE_GLOBAL_CLASS,
                                                       ptr::null_mut(),
                                                       h_option,
                                                       &c_option));
        let _ac = JSAutoCompartment::new(cx, global.get());
        rooted!(in(cx) let prototype_proto = JS_GetObjectPrototype(cx, global.handle().into()));
        rooted!(in(cx) let proto = JS_NewObjectWithUniqueType(cx,
                                                              &CLASS as *const _,
                                                              prototype_proto.handle().into()));
        define_methods(cx, proto.handle(), METHODS).unwrap();
    }
}

#[test]
fn type_rooting() {
    let runtime = Runtime::new().unwrap();
    let cx = runtime.cx();

    rooted!(in(cx) let root : JSVal);
    assert_eq!(root.get().is_undefined(),true);
}

unsafe extern "C" fn generic_method(_: *mut JSContext, _: u32, _: *mut Value) -> bool {
    true
}

const METHODS: &'static [JSFunctionSpec] = &[
    JSFunctionSpec {
        name: b"addEventListener\0" as *const u8 as *const libc::c_char,
        call: JSNativeWrapper { op: Some(generic_method), info: 0 as *const _ },
        nargs: 2,
        flags: JSPROP_ENUMERATE as u16,
        selfHostedName: 0 as *const libc::c_char
    },
    JSFunctionSpec {
        name: b"removeEventListener\0" as *const u8 as *const libc::c_char,
        call: JSNativeWrapper { op: Some(generic_method), info: 0 as *const _  },
        nargs: 2,
        flags: JSPROP_ENUMERATE as u16,
        selfHostedName: 0 as *const libc::c_char
    },
    JSFunctionSpec {
        name: b"dispatchEvent\0" as *const u8 as *const libc::c_char,
        call: JSNativeWrapper { op: Some(generic_method), info: 0 as *const _  },
        nargs: 1,
        flags: JSPROP_ENUMERATE as u16,
        selfHostedName: 0 as *const libc::c_char
    },
    JSFunctionSpec::ZERO,
];

static CLASS: JSClass = JSClass {
    name: b"EventTargetPrototype\0" as *const u8 as *const libc::c_char,
    flags: 0,
    cOps: 0 as *const _,
    reserved: [0 as *mut _; 3]
};
