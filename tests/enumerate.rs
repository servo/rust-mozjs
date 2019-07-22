/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate mozjs;

use mozjs::glue::RUST_JSID_IS_STRING;
use mozjs::glue::RUST_JSID_TO_STRING;
use mozjs::jsapi::GetPropertyKeys;
use mozjs::jsapi::JSITER_OWNONLY;
use mozjs::jsapi::JS_NewGlobalObject;
use mozjs::jsapi::JS_StringEqualsAscii;
use mozjs::jsapi::OnNewGlobalHookOption;
use mozjs::jsval::UndefinedValue;
use mozjs::rust::IdVector;
use mozjs::rust::JSEngine;
use mozjs::rust::RealmOptions;
use mozjs::rust::Runtime;
use mozjs::rust::SIMPLE_GLOBAL_CLASS;
use std::ptr;

#[test]
fn enumerate() {
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

        rooted!(in(cx) let mut rval = UndefinedValue());
        assert!(rt.evaluate_script(global.handle(), "({ 'a': 7 })",
                                   "test", 1, rval.handle_mut()).is_ok());
        assert!(rval.is_object());

        rooted!(in(cx) let object = rval.to_object());
        let ids = IdVector::new(cx);
        assert!(GetPropertyKeys(cx, object.handle().into(), JSITER_OWNONLY, ids.get()));

        assert_eq!(ids.len(), 1);
        rooted!(in(cx) let id = ids[0]);

        assert!(RUST_JSID_IS_STRING(id.handle().into()));
        rooted!(in(cx) let id = RUST_JSID_TO_STRING(id.handle().into()));

        let mut matches = false;
        assert!(JS_StringEqualsAscii(cx,
                                     id.get(),
                                     b"a\0" as *const _ as *const _,
                                     &mut matches));
        assert!(matches);
    }
}
