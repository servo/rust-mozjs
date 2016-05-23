/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate js;

use js::jsapi::CompartmentOptions;
use js::jsapi::JS_Init;
use js::jsapi::JS_NewGlobalObject;
use js::jsapi::OnNewGlobalHookOption;
use js::jsapi::Rooted;
use js::jsapi::RootedValue;
use js::jsval::UndefinedValue;
use js::rust::{Runtime, SIMPLE_GLOBAL_CLASS};

use std::ptr;

#[test]
fn stack_limit() {
    unsafe {
        assert!(JS_Init());

        let rt = Runtime::new(ptr::null_mut());
        let cx = rt.cx();

        let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
        let c_option = CompartmentOptions::default();
        let global = JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS,
                                        ptr::null_mut(), h_option, &c_option);
        let global_root = Rooted::new(cx, global);
        let global = global_root.handle();
        let mut rval = RootedValue::new(cx, UndefinedValue());
        assert!(rt.evaluate_script(global, "function f() { f.apply() } f()",
                                   "test", 1, rval.handle_mut()).is_err());
    }
}
