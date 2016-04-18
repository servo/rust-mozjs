/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate js;
extern crate libc;

use js::jsapi::CompartmentOptions;
use js::jsapi::JSAutoRequest;
use js::jsapi::JS_Init;
use js::jsapi::JS_NewGlobalObject;
use js::jsapi::OnNewGlobalHookOption;
use js::jsapi::RootedObject;
use js::rust::{Runtime, SIMPLE_GLOBAL_CLASS};

use std::ptr;

#[test]
fn evaluate() {
    unsafe { assert!(JS_Init()); }
    let rt = Runtime::new();
    let cx = rt.cx();
    let _ar = JSAutoRequest::new(cx);

    let global = RootedObject::new(cx, unsafe {
        JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(),
                           OnNewGlobalHookOption::FireOnNewGlobalHook,
                           &CompartmentOptions::default())
    });
    assert!(rt.evaluate_script(global.handle(), "1 + 1".to_owned(),
                               "test".to_owned(), 1).is_ok());
}
