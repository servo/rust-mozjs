/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate mozjs;

use mozjs::jsapi::CompartmentOptions;
use mozjs::jsapi::JS_NewGlobalObject;
use mozjs::jsapi::OnNewGlobalHookOption;
use mozjs::jsval::UndefinedValue;
use mozjs::rust::{Runtime, SIMPLE_GLOBAL_CLASS};

use std::ptr;

#[test]
fn evaluate() {
    let rt = Runtime::new().unwrap();
    let cx = rt.cx();

    unsafe {

        rooted!(in(cx) let global =
            JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(),
                               OnNewGlobalHookOption::FireOnNewGlobalHook,
                               &CompartmentOptions::default())
        );
        rooted!(in(cx) let mut rval = UndefinedValue());
        assert!(rt.evaluate_script(global.handle(), "1 + 1",
                                   "test", 1, rval.handle_mut()).is_ok());
    }
}
