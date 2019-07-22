/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate mozjs;

use mozjs::jsapi::JSAutoRealm;
use mozjs::jsapi::JSObject;
use mozjs::jsapi::JS_NewGlobalObject;
use mozjs::jsapi::OnNewGlobalHookOption;
use mozjs::rust::JSEngine;
use mozjs::rust::RealmOptions;
use mozjs::rust::Runtime;
use mozjs::rust::SIMPLE_GLOBAL_CLASS;
use mozjs::typedarray::{CreateWith, Uint32Array};
use std::ptr;

#[test]
#[should_panic]
fn typedarray_update_panic() {
    let engine = JSEngine::init().unwrap();
    let rt = Runtime::new(engine);
    let cx = rt.cx();
    let options = RealmOptions::default();

    unsafe {
        rooted!(in(cx) let global =
            JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(),
                               OnNewGlobalHookOption::FireOnNewGlobalHook,
                               &*options)
        );

        let _ac = JSAutoRealm::new(cx, global.get());
        rooted!(in(cx) let mut rval = ptr::null_mut::<JSObject>());
        let _ = Uint32Array::create(cx, CreateWith::Slice(&[1, 2, 3, 4, 5]), rval.handle_mut());
        typedarray!(in(cx) let mut array: Uint32Array = rval.get());
        array.as_mut().unwrap().update(&[0, 2, 4, 6, 8, 10]);
    }
}
