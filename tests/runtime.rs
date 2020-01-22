/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate mozjs;
extern crate libc;

use mozjs::jsapi::JSAutoRealm;
use mozjs::jsapi::JSClass;
use mozjs::jsapi::JSCLASS_FOREGROUND_FINALIZE;
use mozjs::jsapi::JSClassOps;
use mozjs::jsapi::JSFreeOp;
use mozjs::jsapi::JS_NewGlobalObject;
use mozjs::jsapi::JS_NewObject;
use mozjs::jsapi::JSObject;
use mozjs::jsapi::OnNewGlobalHookOption;
use mozjs::rust::{JSEngine, RealmOptions, Runtime, SIMPLE_GLOBAL_CLASS};
use std::ptr;
use std::thread;
use std::sync::mpsc::channel;

#[test]
fn runtime() {
    let engine = JSEngine::init().unwrap();
    assert!(JSEngine::init().is_err());
    let runtime = Runtime::new(engine.handle());
    unsafe {
        let cx = runtime.cx();
        let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
        let c_option = RealmOptions::default();

        rooted!(in(cx) let global = JS_NewGlobalObject(cx,
                                                       &SIMPLE_GLOBAL_CLASS,
                                                       ptr::null_mut(),
                                                       h_option,
                                                       &*c_option));
        let _ac = JSAutoRealm::new(cx, global.get());
        rooted!(in(cx) let _object = JS_NewObject(cx, &CLASS as *const _));
    }

    let parent = runtime.prepare_for_new_child();
    let (sender, receiver) = channel();
    thread::spawn(move || {
        let runtime = unsafe { Runtime::create_with_parent(parent) };
        assert!(!Runtime::get().is_null());
        drop(runtime);
        let _ = sender.send(());
    });
    let _ = receiver.recv();
}

unsafe extern fn finalize(_fop: *mut JSFreeOp, _object: *mut JSObject) {
    assert!(!Runtime::get().is_null());
}

static CLASS_OPS: JSClassOps = JSClassOps {
    addProperty: None,
    delProperty: None,
    enumerate: None,
    newEnumerate: None,
    resolve: None,
    mayResolve: None,
    finalize: Some(finalize),
    call: None,
    hasInstance: None,
    construct: None,
    trace: None,
};

static CLASS: JSClass = JSClass {
    name: b"EventTargetPrototype\0" as *const u8 as *const libc::c_char,
    flags: JSCLASS_FOREGROUND_FINALIZE,
    cOps: &CLASS_OPS as *const JSClassOps,
    spec: ptr::null(),
    ext: ptr::null(),
    oOps: ptr::null(),
};
