/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate mozjs;

use std::ptr;

use mozjs::glue::{RUST_JSID_IS_STRING, RUST_JSID_TO_STRING};
use mozjs::jsapi::{GetPropertyKeys, JS_NewGlobalObject, JS_StringEqualsAscii, OnNewGlobalHookOption};
use mozjs::jsapi::JSITER_OWNONLY;
use mozjs::jsval::UndefinedValue;
use mozjs::rust::{IdVector, JSEngine, RealmOptions, Runtime, SIMPLE_GLOBAL_CLASS};

#[test]
fn enumerate() {
    let engine = JSEngine::init().unwrap();
    let runtime = Runtime::new(engine.handle());
    let context = runtime.cx();
    let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
    let c_option = RealmOptions::default();

    unsafe {
        rooted!(in(context) let global = JS_NewGlobalObject(
            context,
            &SIMPLE_GLOBAL_CLASS,
            ptr::null_mut(),
            h_option,
            &*c_option,
        ));

        rooted!(in(cx) let mut rval = UndefinedValue());
        assert!(rt
            .evaluate_script(
                global.handle(),
                "({ 'a': 7 })",
                "test",
                1,
                rval.handle_mut()
            )
            .is_ok());
        assert!(rval.is_object());

        rooted!(in(cx) let object = rval.to_object());
        let mut ids = IdVector::new(cx);
        assert!(GetPropertyKeys(
            cx,
            object.handle().into(),
            JSITER_OWNONLY,
            ids.handle_mut(),
        ));

        assert_eq!(ids.len(), 1);
        rooted!(in(cx) let id = ids[0]);

        assert!(RUST_JSID_IS_STRING(id.handle().into()));
        rooted!(in(cx) let id = RUST_JSID_TO_STRING(id.handle().into()));

        let mut matches = false;
        assert!(JS_StringEqualsAscii(
            cx,
            id.get(),
            b"a\0" as *const _ as *const _,
            &mut matches
        ));
        assert!(matches);
    }
}
